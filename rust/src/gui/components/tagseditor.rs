use std::{cmp::Ordering, collections::HashSet};

use gtk::prelude::*;
use relm4::prelude::*;

use crate::gui::util;

/// Component for editing a set of tags.
pub struct TagsEditor {
    editable: bool,
    tags_string_list: gtk::StringList,
    new_tag_buffer: gtk::EntryBuffer,
}

#[derive(Debug)]
pub enum Input {
    Editable(bool),
    SetTags(Option<HashSet<String>>),
    TagAdded(String),
    TagRemoved(String),
    AcceptedRequestTagAdded,
    // Internal:
    SignalAddEnteredTag,
    SignalRemoveTag(String),
}

#[derive(Debug)]
pub enum Output {
    RequestTagAdded(String),
    RequestTagRemoved(String),
}

// TODO gtk::StringList should be useful
// TODO https://gtk-rs.org/gtk4-rs/stable/latest/book/list_widgets.html has some handy guidance
// TODO ListBox or FlowBox - flowbox might be nice if it can be made intuitive

#[relm4::component(pub)]
impl SimpleComponent for TagsEditor {
    type Input = Input;
    type Output = Output;
    type Init = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,

            gtk::ScrolledWindow {
                #[name = "tag_list"]
                gtk::FlowBox,
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,
                #[watch]
                set_sensitive: model.editable,

                gtk::Label::new(Some("Add tag:")) {},

                #[name = "new_tag_entry"]
                gtk::Entry {
                    set_halign: gtk::Align::Start,
                    set_hexpand: true,
                    set_buffer: &model.new_tag_buffer,
                    #[watch]
                    set_sensitive: model.editable,

                    connect_activate[sender] => move |_| {
                        sender.input(Input::SignalAddEnteredTag);
                    },
                },

                #[name = "add_button"]
                gtk::Button::from_icon_name("list-add-symbolic") {
                    #[watch]
                    set_sensitive: model.editable,

                    connect_clicked[sender] => move |_| {
                        sender.input(Input::SignalAddEnteredTag);
                    },
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // TODO empty list initially
        let tags_string_list = gtk::StringList::new(&["foo", "bar", "baz"]);

        let model = TagsEditor {
            editable: true,
            tags_string_list: tags_string_list.clone(),
            new_tag_buffer: gtk::EntryBuffer::builder().max_length(100).build(),
        };

        let widgets = view_output!();

        widgets
            .tag_list
            .bind_model(Some(&tags_string_list), move |object| {
                let string_object = object
                    .downcast_ref::<gtk::StringObject>()
                    .expect("expected a GtkStringObject");

                let remove_button = gtk::Button::from_icon_name("edit-delete-symbolic");
                {
                    let string_object = string_object.clone();
                    let sender = sender.clone();
                    remove_button.connect_clicked(move |_| {
                        sender.input(Input::SignalRemoveTag(string_object.string().to_string()));
                    });
                }
                let label = gtk::Label::builder().build();
                let item_box = gtk::Box::builder()
                    .orientation(gtk::Orientation::Horizontal)
                    .build();
                item_box.append(&remove_button);
                item_box.append(&label);

                string_object
                    .bind_property("string", &label, "label")
                    .sync_create()
                    .build();
                item_box.into()
            });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            Input::Editable(editable) => {
                self.editable = editable;
            }
            Input::SetTags(tag_set) => {
                self.tags_string_list
                    .splice(0, self.tags_string_list.n_items(), &[]);
                if let Some(tag_set) = tag_set {
                    let mut tags = tag_set.into_iter().collect::<Vec<_>>();
                    tags.sort();
                    self.tags_string_list.extend(tags);
                }
            }
            Input::TagAdded(tag) => {
                // Insert to maintain order.
                let mut before_index: Option<u32> = None;
                for i in 0..self.tags_string_list.n_items() {
                    if let Some(s) = self.tags_string_list.string(i) {
                        match tag.as_str().cmp(s.as_str()) {
                            Ordering::Less => {
                                // Insert before this item.
                                before_index = Some(i);
                                break;
                            }
                            Ordering::Equal => {
                                // Already present.
                                return;
                            }
                            Ordering::Greater => {}
                        }
                    }
                }
                if let Some(before_index) = before_index {
                    self.tags_string_list
                        .splice(before_index, 0, &[tag.as_str()]);
                } else {
                    self.tags_string_list.append(&tag);
                }
            }
            Input::TagRemoved(tag) => {
                let mut existing_index: Option<u32> = None;
                for i in 0..self.tags_string_list.n_items() {
                    if let Some(s) = self.tags_string_list.string(i) {
                        if s == tag {
                            existing_index = Some(i);
                            break;
                        }
                    }
                }
                if let Some(existing_index) = existing_index {
                    self.tags_string_list.splice(existing_index, 1, &[]);
                }
            }
            Input::AcceptedRequestTagAdded => {
                self.new_tag_buffer.set_text("");
            }
            Input::SignalAddEnteredTag => {
                util::send_output_or_log(
                    Output::RequestTagAdded(self.new_tag_buffer.text().to_string()),
                    "user request to add tag",
                    &sender,
                );
            }
            Input::SignalRemoveTag(tag) => {
                util::send_output_or_log(
                    Output::RequestTagRemoved(tag),
                    "user request to remove tag",
                    &sender,
                );
            }
        }
    }
}
