#[cfg(test)]
mod tests;

use std::path::Path;

use anyhow::{Result, anyhow, bail};
use hashbrown::HashMap;
use serde_json::Value;

use crate::{
    Node, NodeResult, intermediates,
    plargs::ArgSet,
    spec_types::pdf::{self, TabulaExtractionMethod},
    specs,
    tabula_wrapper::{self, TabulaExtractionRequest, TabulaExtractor},
};

/// System to extract table(s) from a PDF file using Tabula.
pub struct TabulaPdfExtractTableSystem<'t> {
    // TODO: client handle to talk to a worker thread that can be running on the main thread so
    // that the pipeline's processing doesn't have to be. If this turns out to be necessary, which
    // it might not be.
    tabula: &'t dyn TabulaExtractor,
}

impl<'t> TabulaPdfExtractTableSystem<'t> {
    /// Creates a [TabulaPdfExtractTableSystem] that delegates to the [tabula::TabulaEnv] in order
    /// to perform the extraction.
    pub fn new(tabula: &'t dyn TabulaExtractor) -> Self {
        Self { tabula }
    }

    fn extract_table_group(
        &self,
        pdf_path: &Path,
        group: &ExtractGroupKey,
        node_specs: &[NodeSpec],
    ) -> Result<tabula_wrapper::JsonTableSet> {
        let page_areas: Vec<_> = node_specs
            .iter()
            .map(|node_spec| node_spec.spec.rect.to_tabula_rectangle_page_area())
            .collect();

        self.tabula.extract_tables(TabulaExtractionRequest {
            pdf_path: pdf_path.to_path_buf(),
            password: None,
            page: group.page,
            guess: false,
            // TODO: Decide if newlines should be retained, or if it should be configurable.
            use_returns: false,
            page_areas,
            method: group.method.to_tabula_extraction_method(),
        })
    }

    fn extract_table_group_to_intermediates(
        &self,
        results: &mut Vec<NodeResult>,
        pdf_path: &Path,
        group: &ExtractGroupKey,
        node_specs: &[NodeSpec],
    ) {
        let table_set = match self.extract_table_group(pdf_path, group, node_specs) {
            Ok(table_set) => table_set,
            Err(err) => {
                for node_spec in node_specs {
                    results.push(NodeResult {
                        id: node_spec.node.id.clone(),
                        value: Err(anyhow!("failed to batch extract table: {err:?}")),
                    });
                }
                return;
            }
        };

        let mut table_set_iter = table_set.0.into_iter().peekable();
        for node_spec in node_specs {
            let mut found_table: TableMatch = TableMatch::None;

            // Consume tables within the node_spec's region until one falls outside of it.
            while let Some(table) =
                table_set_iter.next_if(|table| is_json_table_within(table, &node_spec.spec.rect))
            {
                found_table.add_match(table);
            }

            let value: Result<intermediates::IntermediateValue> = match found_table {
                TableMatch::None => Err(anyhow!("no table in region")),
                TableMatch::One(table) => {
                    // TODO: Consider if in future the raw JsonTableSet should be returned, which
                    // could be specifed via an option on the specs.
                    Ok(intermediates::JsonData(convert_tabula_table_to_table_json(table)).into())
                }
                TableMatch::Many(n) => Err(anyhow!("multiple ({n}) tables in region")),
            };

            results.push(NodeResult {
                id: node_spec.node.id.clone(),
                value,
            });
        }
    }

    fn extract_table_group_to_intermediates_without_overlapping(
        &self,
        results: &mut Vec<NodeResult>,
        pdf_path: &Path,
        group: &ExtractGroupKey,
        node_specs: Vec<NodeSpec>,
    ) {
        grouped_non_overlapping_slices(node_specs, |node_specs| {
            self.extract_table_group_to_intermediates(results, pdf_path, group, node_specs)
        });
    }

    fn extract_groups(
        &self,
        intermediates: &generic_pipeline::intermediates::GenericIntermediateSet<
            intermediates::IntermediateValue,
        >,
        results: &mut Vec<generic_pipeline::systems::NodeResult<intermediates::IntermediateValue>>,
        pdf_group_to_node_specs: HashMap<
            generic_pipeline::node::NodeId,
            HashMap<ExtractGroupKey, Vec<NodeSpec<'_>>>,
        >,
    ) {
        for (pdf_id, group_to_node_specs) in pdf_group_to_node_specs {
            // Get path to the PDF for this group of extractions.
            let pdf_path = match intermediates
                .require(&pdf_id)
                .and_then(<&intermediates::InputFile>::try_from)
            {
                Ok(input_file) => &input_file.0,
                Err(err) => {
                    for (_, node_specs) in &group_to_node_specs {
                        for node_spec in node_specs {
                            results.push(NodeResult {
                                id: node_spec.node.id.clone(),
                                value: Err(anyhow!("{err:?}")),
                            });
                        }
                    }
                    continue;
                }
            };

            // Perform the batch extraction.
            for (group, node_specs) in group_to_node_specs {
                self.extract_table_group_to_intermediates_without_overlapping(
                    results, pdf_path, &group, node_specs,
                );
            }
        }
    }
}

impl<'env> generic_pipeline::systems::GenericSystem<crate::PipelineTypes>
    for TabulaPdfExtractTableSystem<'env>
{
    fn inputs<'a>(
        &self,
        node: &Node,
        reg: &'a mut generic_pipeline::plinputs::NodeInputsRegistrator<'a>,
    ) -> Result<()> {
        let spec = <&specs::PdfExtractTable>::try_from(&node.spec)?;
        reg.add_input(&spec.pdf);
        Ok(())
    }

    fn process(
        &self,
        node: &Node,
        args: &ArgSet,
        intermediates: &intermediates::IntermediateSet,
    ) -> Result<<crate::PipelineTypes as generic_pipeline::PipelineTypes>::IntermediateValue> {
        let mut result = self.process_multiple(&[node], args, intermediates);
        if result.len() != 1 {
            bail!(
                "bug: process_multiple did not produce exactly one value, got {}",
                result.len(),
            );
        }
        let result = result.pop().expect("bug: length was checked");
        if result.id != node.id {
            bail!(
                "bug: process_multiple returned result for {:?} but expected result for {:?}",
                result.id,
                node.id
            );
        }
        result.value
    }

    fn process_multiple<'a>(
        &self,
        nodes: &'a [&'a Node],
        _args: &ArgSet,
        intermediates: &intermediates::IntermediateSet,
    ) -> Vec<NodeResult> {
        let mut results = Vec::with_capacity(nodes.len());

        let pdf_group_to_node_specs = group_nodes_for_extraction(nodes, &mut results);

        self.extract_groups(intermediates, &mut results, pdf_group_to_node_specs);

        results
    }
}

/// Groups nodes for extraction by [ExtractGroupKey], to reduce the number of calls into Tabula.
fn group_nodes_for_extraction<'a>(
    nodes: &'a [&'a generic_pipeline::node::GenericNode<specs::Spec>],
    results: &mut Vec<generic_pipeline::systems::NodeResult<intermediates::IntermediateValue>>,
) -> HashMap<generic_pipeline::node::NodeId, HashMap<ExtractGroupKey, Vec<NodeSpec<'a>>>> {
    let mut pdf_group_to_node_specs: HashMap<
        crate::NodeId,
        HashMap<ExtractGroupKey, Vec<NodeSpec>>,
    > = HashMap::new();
    for node in nodes {
        let spec = match <&specs::PdfExtractTable>::try_from(&node.spec) {
            Ok(spec) => spec,
            Err(err) => {
                results.push(NodeResult {
                    id: node.id.clone(),
                    value: Err(err),
                });
                continue;
            }
        };

        pdf_group_to_node_specs
            .entry_ref(&spec.pdf)
            .or_default()
            .entry(ExtractGroupKey {
                page: spec.page,
                method: spec.method,
            })
            .or_default()
            .push(NodeSpec { node, spec });
    }
    pdf_group_to_node_specs
}

struct NodeSpec<'a> {
    node: &'a Node,
    spec: &'a specs::PdfExtractTable,
}

/// Key for grouping together extracted tables from a single PDF file.
///
/// This grouping is to fit multiple table extractions into a single call to
/// [tabula::TabulaEnv::configure_tabula].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct ExtractGroupKey {
    page: i32,
    method: TabulaExtractionMethod,
}

fn is_json_table_within(json_table: &tabula_wrapper::JsonTable, rect: &pdf::TabulaPdfRect) -> bool {
    json_table.top >= rect.top.to_f32()
        && json_table.bottom <= rect.bottom.to_f32()
        && json_table.left >= rect.left.to_f32()
        && json_table.right <= rect.right.to_f32()
}

fn grouped_non_overlapping_slices<C: FnMut(&[NodeSpec])>(
    mut node_specs: Vec<NodeSpec>,
    mut group_callback: C,
) {
    // Group node_specs into non-overlapping groups to extract separately, to avoid ambiguity (at
    // the very least, lattice method can output [0,many] tables).
    let mut group_start: usize = 0;
    while group_start < node_specs.len() {
        let mut group_end = group_start + 1;
        let search_range = group_end..node_specs.len();
        for i in search_range {
            let node_i = &node_specs[i];
            let node_overlaps = node_specs[group_start..group_end]
                .iter()
                .any(|node_in_group| node_in_group.spec.rect.is_overlapping(&node_i.spec.rect));

            // If the node at [i] doesn't overlap anything already in the group, then we can
            // include it in the group.
            if !node_overlaps {
                node_specs.swap(i, group_end);
                group_end += 1;
            }
        }
        group_callback(&node_specs[group_start..group_end]);
        group_start = group_end;
    }
}

fn convert_tabula_table_to_table_json(tabula_table: tabula_wrapper::JsonTable) -> Value {
    Value::Array(
        tabula_table
            .data
            .into_iter()
            .map(|row| {
                Value::Array(
                    row.0
                        .into_iter()
                        .map(|field| Value::String(field.text))
                        .collect(),
                )
            })
            .collect(),
    )
}

enum TableMatch {
    None,
    One(tabula_wrapper::JsonTable),
    Many(usize),
}

impl TableMatch {
    fn add_match(&mut self, table: tabula_wrapper::JsonTable) {
        use TableMatch::*;

        *self = match self {
            None => One(table),
            One(_) => Many(2),
            Many(n) => Many(n.saturating_add(1)),
        }
    }
}
