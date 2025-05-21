use anyhow::{Context, Result, bail};
use gtk::prelude::*;
use relm4::prelude::*;

use crate::{gui::util, templatedoc};

use super::{tagseditor, textentry};

/// GUI component to edit a [templatedoc::Group].
pub struct EditGroup {
    group: Option<templatedoc::Group>,

    name_text_entry: Controller<textentry::TextEntry>,
    tags_editor: Controller<tagseditor::TagsEditor>,
}

#[derive(Debug)]
pub enum Input {
    /// Sets the group to edit, or no group at all.
    SetGroup(Option<templatedoc::Group>),
    // Internal:
    Edit(Edit),
    // TODO: Receive events for outside changes to the group (such as from undo/redo).
}

#[derive(Debug)]
pub enum Edit {
    SetName(String),
    RequestChangeTags(tagseditor::Output),
}

#[derive(Debug)]
pub enum Output {
    Error(String),
}

#[relm4::component(pub)]
impl SimpleComponent for EditGroup {
    type Input = Input;
    type Output = Output;
    type Init = ();

    view! {
        #[root]
        gtk::Grid {
            set_margin_start: 5,
            set_margin_end: 5,
            set_margin_top: 5,
            set_margin_bottom: 5,
            set_column_spacing: 5,
            set_row_spacing: 5,

            attach[0, 0, 1, 1] = &gtk::Label {
                set_label: "Name:",
                set_halign: gtk::Align::Start,
            },
            attach[1, 0, 1, 1] = model.name_text_entry.widget(),

            attach[0, 1, 2, 1] = &gtk::Label {
                set_label: "Tags:",
                set_halign: gtk::Align::Start,
            },
            attach[0, 2, 2, 1] = model.tags_editor.widget(),
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = EditGroup {
            group: None,
            name_text_entry: textentry::TextEntry::builder()
                .launch(None)
                .forward(sender.input_sender(), |name| {
                    Input::Edit(Edit::SetName(name))
                }),
            tags_editor: tagseditor::TagsEditor::builder()
                .launch(())
                .forward(sender.input_sender(), |message| {
                    Input::Edit(Edit::RequestChangeTags(message))
                }),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        let result = match message {
            Input::SetGroup(group) => self.set_group(group),
            Input::Edit(edit) => self.do_edit(edit),
        };
        util::send_if_error_or_log(result, Output::Error, &sender);
    }
}

impl EditGroup {
    fn set_group(&mut self, group: Option<templatedoc::Group>) -> Result<()> {
        // TODO: Subscribe/unsubscribe to group events to receive updates to properties.
        match &group {
            Some(group) => {
                let data = group.get_data().context("getting group data")?;

                self.name_text_entry.emit(Some(data.name));

                self.tags_editor
                    .emit(tagseditor::Input::SetTags(Some(data.tags)));
                self.tags_editor.emit(tagseditor::Input::Editable(true));
            }
            None => {
                self.name_text_entry.emit(None);

                self.tags_editor.emit(tagseditor::Input::SetTags(None));
                self.tags_editor.emit(tagseditor::Input::Editable(false));
            }
        }
        self.group = group;
        Ok(())
    }

    fn do_edit(&mut self, edit: Edit) -> Result<()> {
        let group = if let Some(group) = &self.group {
            group
        } else {
            bail!("Cannot edit group - edited group is not set.");
        };

        use Edit::*;
        match edit {
            SetName(name) => {
                group.edit_name(name).context("editing group name")?;
            }
            RequestChangeTags(message) => {
                use tagseditor::Output::*;
                match message {
                    RequestTagAdded(tag) => {
                        group.add_tag(tag)?;
                        self.tags_editor
                            .emit(tagseditor::Input::AcceptedRequestTagAdded);
                    }
                    RequestTagRemoved(tag) => {
                        group.remove_tag(tag)?;
                    }
                }
            }
        }
        Ok(())
    }
}
