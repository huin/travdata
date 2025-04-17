use anyhow::{Context, Result, bail};
use gtk::prelude::*;
use relm4::prelude::*;

use crate::gui::{tmplmodel, util};

use super::textentry;

/// GUI component to edit a [tmplmodel::Group].
pub struct EditGroup {
    group: Option<tmplmodel::Group>,

    name_text_entry: Controller<textentry::TextEntry>,
}

#[derive(Debug)]
pub enum Input {
    /// Sets the group to edit, or no group at all.
    SetGroup(Option<tmplmodel::Group>),
    // Internal:
    Edit(Edit),
    // TODO: Receive events for outside changes to the group (such as from undo/redo).
}

#[derive(Debug)]
pub enum Edit {
    SetName(String),
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
    fn set_group(&mut self, group: Option<tmplmodel::Group>) -> Result<()> {
        match &group {
            Some(group) => {
                self.name_text_entry
                    .emit(Some(group.get_name().context("getting group name")?));
            }
            None => {
                self.name_text_entry.emit(None);
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

        match edit {
            Edit::SetName(name) => {
                group.edit_name(name).context("editing group name")?;
            }
        }
        Ok(())
    }
}
