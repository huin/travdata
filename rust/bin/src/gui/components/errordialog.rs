use gtk::prelude::*;
use relm4::gtk;
use relm4::prelude::*;

use crate::gui::util;

/// Error messsage dialog box.
pub struct ErrorDialog {
    hidden: bool,
    message: String,
}

/// Input messages for [ErrorDialog].
#[derive(Debug)]
pub enum Input {
    ShowErrorMessage(String),
    // Internal:
    Acknowledged,
}

/// Output messages for [ErrorDialog].
#[derive(Debug)]
pub enum Output {
    Acknowledged,
}

#[relm4::component(pub)]
impl SimpleComponent for ErrorDialog {
    type Input = Input;
    type Output = Output;
    type Init = ();

    view! {
        #[root]
        gtk::MessageDialog {
            set_modal: true,
            #[watch]
            set_visible: !model.hidden,
            set_text: Some("Error"),
            #[watch]
            set_secondary_text: Some(&model.message),
            add_button: ("Acknowledge", gtk::ResponseType::Accept),

            connect_response[sender] => move |_, _| {
                sender.input(Input::Acknowledged);
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            hidden: true,
            message: "".into(),
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            Input::ShowErrorMessage(message) => {
                self.message = message;
                self.hidden = false;
            }
            Input::Acknowledged => {
                self.hidden = true;
                util::send_output_or_log(Output::Acknowledged, "error acknowledge", &sender);
            }
        }
    }
}
