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
    /// [Option::Some] sets the current value and makes it editable, [Option::None] clears the text
    /// entry and makes the text entry insensitive.
    type Init = Option<String>;
    /// Same behaviour as per [Component::Init].
    type Input = Option<String>;
    /// Value set via user action.
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

        set(&buffer, &root, init);
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

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, root: &Self::Root) {
        let notify_id = match &self.notify_id {
            Some(notify_id) => notify_id,
            None => {
                log::warn!("Received new text for TextEntry, but notify_id is not set.");
                return;
            }
        };
        self.buffer.block_signal(notify_id);
        set(&self.buffer, root, message);
        self.buffer.unblock_signal(notify_id);
    }

    fn shutdown(&mut self, _widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {
        if let Some(notify_id) = self.notify_id.take() {
            self.buffer.disconnect(notify_id);
        }
    }
}

fn set(buffer: &gtk::EntryBuffer, entry: &gtk::Entry, value: Option<String>) {
    match value {
        Some(value) => {
            buffer.set_text(value);
            entry.set_sensitive(true);
        }
        None => {
            buffer.set_text("");
            entry.set_sensitive(false);
        }
    }
}
