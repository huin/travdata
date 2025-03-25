use gtk::prelude::*;
use relm4::prelude::*;
use relm4_components::simple_combo_box;

use crate::{gui::util, template::serialised};

pub struct TemplateVersionDialog {
    hidden: bool,
    version_selector: Controller<simple_combo_box::SimpleComboBox<&'static str>>,

    version_index: usize,
}

#[derive(Debug)]
pub enum Input {
    RequestSelectVersion,
    // Internal:
    ImportClicked,
    CancelClicked,
    VersionSelected(usize),
}

#[derive(Debug)]
pub enum Output {
    VersionResponse(&'static str),
    Cancelled,
}

#[relm4::component(pub)]
impl SimpleComponent for TemplateVersionDialog {
    type Input = Input;
    type Output = Output;
    type Init = ();

    view! {
        #[root]
        gtk::Dialog {
            #[watch]
            set_visible: !model.hidden,

            gtk::Grid {
                attach[0, 0, 1, 1] = &gtk::Label {
                    set_label: "Template version:",
                    set_halign: gtk::Align::Start,
                },
                attach[1, 0, 1, 1] = model.version_selector.widget(),

                attach[0, 1, 2, 1] = &gtk::Box {
                    gtk::Button::with_label("Import") {
                        connect_clicked[sender] => move |_| {
                            sender.input(Input::ImportClicked);
                        }
                    },
                    gtk::Button::with_label("Cancel") {
                        connect_clicked[sender] => move |_| {
                            sender.input(Input::CancelClicked);
                        }
                    },
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let variants = serialised::VERSIONS_SUPPORTED.iter().map(|s| *s).collect();

        let model = TemplateVersionDialog {
            hidden: true,
            version_selector: simple_combo_box::SimpleComboBox::builder()
                .launch(simple_combo_box::SimpleComboBox {
                    variants,
                    active_index: Some(0),
                })
                .forward(sender.input_sender(), Input::VersionSelected),

            version_index: 0,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            Input::RequestSelectVersion => {
                self.hidden = false;
            }
            Input::ImportClicked => {
                if let Some(version) = serialised::VERSIONS_SUPPORTED.get(self.version_index) {
                    util::send_output_or_log(
                        Output::VersionResponse(version),
                        "template version confirmation",
                        &sender,
                    );
                    self.hidden = true;
                }
            }
            Input::CancelClicked => {
                util::send_output_or_log(
                    Output::Cancelled,
                    "template version cancellation",
                    &sender,
                );
                self.hidden = true;
            }
            Input::VersionSelected(index) => {
                self.version_index = index;
            }
        }
    }
}
