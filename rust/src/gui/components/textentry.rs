use gtk::{glib, prelude::*};
use relm4::prelude::*;

use crate::gui::util;

/// Component for a [gtk::Entry].
pub struct TextEntry {
    buffer: gtk::EntryBuffer,
    notify_id: Option<glib::signal::SignalHandlerId>,
}

impl Component for TextEntry {
    type CommandOutput = ();
    type Init = String;
    type Input = String;
    type Output = String;
    type Root = gtk::Entry;
    type Widgets = gtk::Entry;

    fn init_root() -> Self::Root {
        gtk::Entry::default()
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let buffer = root.buffer();
        buffer.set_text(init);
        let notify_id = buffer.connect_text_notify(move |buffer| {
            let s = buffer.text().to_string();
            util::send_output_or_log(s, "text entry content", &sender);
        });

        let model = Self {
            buffer,
            notify_id: Some(notify_id),
        };

        let widgets = root;

        ComponentParts { model, widgets }
    }

    fn builder() -> relm4::ComponentBuilder<Self> {
        relm4::ComponentBuilder::<Self>::default()
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        self.buffer.set_text(message);
    }

    fn shutdown(&mut self, _widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {
        if let Some(notify_id) = self.notify_id.take() {
            self.buffer.disconnect(notify_id);
        }
    }
}
