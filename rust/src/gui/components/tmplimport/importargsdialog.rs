use gtk::prelude::*;
use relm4::prelude::*;
use relm4_components::simple_combo_box;

use crate::{gui::util, template::serialised};

#[derive(Debug)]
pub enum Input {
    RequestLoadArgs(serialised::PreloadData),
    // Internal:
    ImportClicked,
    CancelClicked,
    BookSelected(usize),
}

#[derive(Debug)]
pub enum Output {
    LoadArg(serialised::LoadArg),
    Cancelled,
}

pub struct ImportArgsDialog {
    hidden: bool,
    book_selector: Controller<simple_combo_box::SimpleComboBox<serialised::BookIdName>>,

    need_book_id: bool,
    book_id: Option<String>,
}

impl ImportArgsDialog {
    fn is_valid(&self) -> bool {
        !self.need_book_id || self.book_id.is_some()
    }

    fn take_load_args(&mut self) -> serialised::LoadArg {
        let book_id = self.book_id.take();
        serialised::LoadArg { book_id }
    }
}

#[relm4::component(pub)]
impl SimpleComponent for ImportArgsDialog {
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
                    set_label: "Book:",
                    set_halign: gtk::Align::Start,
                },
                attach[1, 0, 1, 1] = model.book_selector.widget(),

                attach[0, 1, 2, 1] = &gtk::Box {
                    gtk::Button::with_label("Import") {
                        #[watch]
                        set_sensitive: model.is_valid(),
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
        let model = ImportArgsDialog {
            hidden: true,
            book_selector: simple_combo_box::SimpleComboBox::builder()
                .launch(simple_combo_box::SimpleComboBox {
                    variants: vec![],
                    active_index: None,
                })
                .forward(sender.input_sender(), Input::BookSelected),

            need_book_id: false,
            book_id: None,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            Input::RequestLoadArgs(preload_data) => {
                self.need_book_id = preload_data.book_ids.is_some();

                let variants: Vec<serialised::BookIdName> =
                    preload_data.book_ids.unwrap_or_default();
                self.book_selector
                    .emit(simple_combo_box::SimpleComboBoxMsg::UpdateData(
                        simple_combo_box::SimpleComboBox {
                            variants,
                            active_index: None,
                        },
                    ));

                self.hidden = false;
            }
            Input::ImportClicked => {
                util::send_output_or_log(
                    Output::LoadArg(self.take_load_args()),
                    "import args confirmation",
                    &sender,
                );
                self.hidden = true;
            }
            Input::CancelClicked => {
                util::send_output_or_log(Output::Cancelled, "import args cancellation", &sender);
                self.hidden = true;
            }
            Input::BookSelected(index) => {
                if let Some(book_entry) = self.book_selector.model().variants.get(index) {
                    self.book_id = Some(book_entry.id.clone());
                }
            }
        }
    }
}
