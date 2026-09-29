use std::sync::Arc;

use crate::app::{
    colours,
    components::{WidgetState as _, WidgetStateTemporary},
    data::{self, guinode},
    icons,
};

pub struct NodeIdRefEditor<'n, 'idx> {
    gui_node_id: &'n mut guinode::NodeIdRef,
    node_index: &'idx data::NodeIndex,
}

impl<'n, 'idx> egui::Widget for NodeIdRefEditor<'n, 'idx> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        self.show(ui)
    }
}

/// Component to edit a reference to a node.
impl<'n, 'idx> NodeIdRefEditor<'n, 'idx> {
    pub fn new(gui_node_id: &'n mut guinode::NodeIdRef, node_index: &'idx data::NodeIndex) -> Self {
        Self {
            gui_node_id,
            node_index,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let mut state = State::get_or_default(ui);

        let response = ui.horizontal(|ui| {
            let (drop_zone_response, dragged_node_ref) = ui.dnd_drop_zone::<guinode::NodeRef, _>(
                egui::Frame::default().inner_margin(4.0),
                |ui| {
                    icons::node_ref_dest(ui, self.gui_node_id.is_resolved());
                    self.search_button(ui)
                },
            );
            drop_zone_response
                .response
                .on_hover_cursor(egui::CursorIcon::Grab)
                .on_hover_and_drag_cursor(egui::CursorIcon::Grabbing);

            // Handle starting/stopping a search from a click on the search button.
            let (mut button_response, initial_query) = drop_zone_response.inner;
            button_response = button_response
                .on_hover_text("Reference to another node, drag a node ID here to reference it.");
            if button_response.clicked() {
                state.search = if state.search.is_none() {
                    Some(Search {
                        query: initial_query.to_string(),
                    })
                } else {
                    None
                };
            }

            // Handle DND drop onto the search button.
            if let Some(dragged_node_ref) = dragged_node_ref {
                *self.gui_node_id = guinode::NodeIdRef::Resolved(*dragged_node_ref)
            }

            let popup_response = if let Some(search) = &mut state.search {
                self.search_popup(&button_response, search)
            } else {
                None
            };

            if let Some(popup_response) = popup_response
                && popup_response.response.should_close()
            {
                state.search = None;
            }

            button_response
        });

        state.store(ui);

        response.inner
    }

    fn search_button<'a>(&'a self, ui: &mut egui::Ui) -> (egui::Response, &'a str) {
        use guinode::NodeIdRef;

        let (button_text, initial_search_text): (&str, &str) = match &self.gui_node_id {
            NodeIdRef::Unresolved(node_id) => (node_id, node_id),
            NodeIdRef::Resolved(node_ref) => match self.node_index.lookup_node_ref(*node_ref) {
                Some(node_index_entry) => {
                    let node_id = node_index_entry.node_id();
                    (node_id, node_id)
                }
                None => ("<deleted node>", ""),
            },
        };

        (
            ui.button(egui::RichText::new(button_text).color(colours::DRAGGABLE_NODE_ID)),
            initial_search_text,
        )
    }

    fn search_popup(
        self,
        button_response: &egui::Response,
        search: &mut Search,
    ) -> Option<egui::InnerResponse<egui::InnerResponse<()>>> {
        egui::Popup::menu(button_response)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show(|ui| {
                ui.label("Node search");
                let query_response = ui.add(
                    egui::TextEdit::singleline(&mut search.query)
                        .hint_text("Search node ID by prefix")
                        .cursor_at_end(true),
                );
                if button_response.clicked() {
                    query_response.request_focus();
                }

                if search.query.is_empty() {
                    ui.push_id("empty_prompt", |ui| {
                        ui.label("Please enter a node ID prefix.");
                    })
                } else {
                    ui.push_id("results", |ui| {
                        let search_result = Searcher::search(ui, &search.query, self.node_index);

                        for result in search_result.results.iter() {
                            if ui.button(result.node_id()).clicked() {
                                *self.gui_node_id =
                                    guinode::NodeIdRef::Resolved(*result.node_ref());

                                ui.close();
                            }
                        }

                        if let Some(num_extra_message) = &search_result.num_extra_message {
                            ui.label(num_extra_message);
                        }
                    })
                }
            })
    }
}

#[derive(Clone, Default)]
struct State {
    search: Option<Search>,
}

#[derive(Clone, Default)]
struct Search {
    query: String,
}

#[derive(Default)]
struct Searcher;

impl Searcher {
    const MAX_RESULTS: usize = 10;

    fn search(ui: &mut egui::Ui, query: &str, node_index: &data::NodeIndex) -> Arc<SearchResult> {
        ui.memory_mut(|mem| {
            mem.caches
                .cache::<SearchCache>()
                .get((query, node_index.generation(), NodeIndexRef(node_index)))
                .clone()
        })
    }
}

impl<'a>
    egui::cache::ComputerMut<(&str, data::NodeIndexGeneration, NodeIndexRef<'a>), Arc<SearchResult>>
    for Searcher
{
    fn compute(
        &mut self,
        key: (&str, data::NodeIndexGeneration, NodeIndexRef),
    ) -> Arc<SearchResult> {
        let (query, _index_generation, index) = key;

        let mut results_iter = index.0.scan_node_id_prefix(query).enumerate();
        let mut results = Vec::with_capacity(Self::MAX_RESULTS);
        for (i, result) in &mut results_iter {
            if i >= (Self::MAX_RESULTS - 1) {
                break;
            }
            results.push(result.clone());
        }

        let num_extra = results_iter.count();

        Arc::new(SearchResult {
            results,
            num_extra_message: if num_extra == 0 {
                None
            } else {
                Some(format!("And {num_extra} matching nodes..."))
            },
        })
    }
}

type SearchCache = egui::cache::FrameCache<Arc<SearchResult>, Searcher>;

#[derive(Clone, Copy)]
struct NodeIndexRef<'a>(&'a data::NodeIndex);

impl<'a> std::hash::Hash for NodeIndexRef<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // We only need the hash of the NodeIndex's identity for a useful (and cheap) cache key. The
        // NodeIndexGeneration covers index content changes.
        let index_ptr: *const data::NodeIndex = self.0;
        index_ptr.hash(state);
    }
}

struct SearchResult {
    results: Vec<data::NodeIndexEntry>,
    num_extra_message: Option<String>,
}

impl WidgetStateTemporary for State {}
