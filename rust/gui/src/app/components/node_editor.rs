use egui_form::{
    Form,
    validator::{ValidatorReport, field_path},
};

use crate::app::{
    components::{node_ref_editor::NodeIdRefEditor, todo_ui},
    data::{self, guinode},
    validate::FormFieldFactory as _,
};

pub struct NodeEditor<'node_ctx, 'node_ref> {
    node_ctx: &'node_ctx mut data::NodeContextMut<'node_ref>,
}

impl<'node_ref, 'node> egui::Widget for NodeEditor<'node_ref, 'node> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        self.show(ui);
        ui.response()
    }
}

impl<'node_ctx, 'node_ref> NodeEditor<'node_ctx, 'node_ref>
where
    'node_ref: 'node_ctx,
{
    pub fn new(node_ctx: &'node_ctx mut data::NodeContextMut<'node_ref>) -> Self {
        Self { node_ctx }
    }

    pub fn show(mut self, ui: &mut egui::Ui) {
        ui.push_id(("NodeEditor", self.node_ctx.node.meta.self_ref), |ui| {
            self.node_meta_editor(ui);
            self.node_spec_editor(ui);
        });
    }

    fn node_meta_editor(&mut self, ui: &mut egui::Ui) {
        let mut form = Form::new().add_report(ValidatorReport::validate(&self.node_ctx.node.meta));
        if form
            .field(field_path!("id"))
            .label("ID")
            .ui(
                ui,
                egui::TextEdit::singleline(&mut self.node_ctx.node.meta.id),
            )
            .changed()
        {
            self.node_ctx.mark_node_changed();
        }
        ui.end_row();
    }

    fn node_spec_editor(&mut self, ui: &mut egui::Ui) {
        use guinode::Spec;

        match &mut self.node_ctx.node.spec {
            Spec::InputPdfFile(spec) => {
                let mut form = Form::new().add_report(ValidatorReport::validate(&*spec));
                if form
                    .field(field_path!("descripion"))
                    .label("PDF description")
                    .ui(ui, egui::TextEdit::multiline(&mut spec.description))
                    .changed()
                {
                    self.node_ctx.mark_node_changed();
                }
            }
            Spec::JsContext(_spec) => {
                // No fields yet.
                ui.label("No settings for JsContext yet.");
            }
            Spec::JsTransform(spec) => {
                let mut form = Form::new().add_report(ValidatorReport::validate(&*spec));

                form.field(field_path!("context", "self"))
                    .label("Context")
                    .ui(
                        ui,
                        NodeIdRefEditor::new(&mut spec.context, self.node_ctx.node_index),
                    );

                ui.separator();
                ui.label("Inputs");
                {
                    if ui.button("New input").clicked() {
                        spec.input_data.push(guinode::JsTransformParam {
                            name: "new_input".into(),
                            input: guinode::NodeIdRef::Unresolved("".into()),
                        });
                    }

                    let text_height = egui::TextStyle::Body
                        .resolve(ui.style())
                        .size
                        .max(ui.spacing().interact_size.y);
                    let available_height = ui.available_height();

                    let mut remove_indices = Vec::with_capacity(0);
                    egui_extras::TableBuilder::new(ui)
                        .striped(true)
                        .column(egui_extras::Column::auto_with_initial_suggestion(100.0))
                        .column(egui_extras::Column::auto_with_initial_suggestion(100.0))
                        .column(egui_extras::Column::auto_with_initial_suggestion(60.0))
                        .resizable(true)
                        .min_scrolled_height(10.0)
                        .max_scroll_height(available_height)
                        .header(20.0, |mut header| {
                            header.col(|ui| {
                                ui.strong("Name");
                            });
                            header.col(|ui| {
                                ui.strong("Value");
                            });
                            header.col(|ui| {
                                ui.strong("Remove");
                            });
                        })
                        .body(|body| {
                            body.rows(text_height, spec.input_data.len(), |mut row| {
                                let index = row.index();
                                let param = match spec.input_data.get_mut(index) {
                                    Some(param) => param,
                                    None => return,
                                };

                                row.col(|ui| {
                                    form.field(field_path!("inputs", index, "name"))
                                        .ui(ui, egui::TextEdit::singleline(&mut param.name));
                                });

                                row.col(|ui| {
                                    ui.add(NodeIdRefEditor::new(
                                        &mut param.input,
                                        self.node_ctx.node_index,
                                    ));
                                });

                                row.col(|ui| {
                                    if ui.button("Remove").clicked() {
                                        remove_indices.push(index);
                                    }
                                });
                            });
                        });

                    remove_indices.reverse();
                    for index in remove_indices.into_iter() {
                        spec.input_data.remove(index);
                    }
                }

                ui.separator();

                let response = ui.vertical(|ui| {
                    ui.code("function(inputs) {");
                    let response = form
                        .field(field_path!("code"))
                        .label("Code")
                        .ui(ui, egui::TextEdit::multiline(&mut spec.code).code_editor());
                    ui.code("}");
                    response
                });
                if response.inner.changed() {
                    self.node_ctx.mark_node_changed();
                }
            }
            Spec::OutputDirectory(spec) => {
                let mut form = Form::new().add_report(ValidatorReport::validate(&*spec));
                if form
                    .field(field_path!("description"))
                    .label("Directory description")
                    .ui(ui, egui::TextEdit::multiline(&mut spec.description))
                    .changed()
                {
                    self.node_ctx.mark_node_changed();
                }
            }
            Spec::OutputFileCsv(spec) => {
                let mut form = Form::new().add_report(ValidatorReport::validate(&*spec));
                form.field(field_path!("input_data"))
                    .label("Input data")
                    .ui(
                        ui,
                        NodeIdRefEditor::new(&mut spec.input_data, self.node_ctx.node_index),
                    );

                form.field(field_path!("directory")).label("Directory").ui(
                    ui,
                    NodeIdRefEditor::new(&mut spec.directory, self.node_ctx.node_index),
                );

                form.field(field_path!("filename", "self"))
                    .label("Filename")
                    .ui(ui, egui::Label::new(&spec.filename.0));
            }
            Spec::OutputFileJson(spec) => {
                let mut form = Form::new().add_report(ValidatorReport::validate(&*spec));
                form.field(field_path!("input_data"))
                    .label("Input data")
                    .ui(
                        ui,
                        NodeIdRefEditor::new(&mut spec.input_data, self.node_ctx.node_index),
                    );

                form.field(field_path!("directory")).label("Directory").ui(
                    ui,
                    NodeIdRefEditor::new(&mut spec.directory, self.node_ctx.node_index),
                );

                form.field(field_path!("filename", "self"))
                    .label("Filename")
                    .ui(ui, egui::Label::new(&spec.filename.0));
            }
            Spec::PdfExtractTable(spec) => {
                let mut form = Form::new().add_report(ValidatorReport::validate(&*spec));
                form.field(field_path!("pdf")).label("PDF").ui(
                    ui,
                    NodeIdRefEditor::new(&mut spec.pdf, self.node_ctx.node_index),
                );

                form.field(field_path!("page")).label("Page").ui(
                    ui,
                    egui::DragValue::new(&mut spec.page)
                        .speed(1)
                        .range(1..=i32::MAX),
                );

                todo_ui(ui, "Other fields.");
            }
        }
    }
}
