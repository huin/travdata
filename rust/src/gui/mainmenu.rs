use gtk::Application;
use gtk::prelude::*;
use relm4::{
    actions::{ActionName, EmptyType, RelmAction, RelmActionGroup},
    menu, new_action_group, new_stateless_action,
    prelude::*,
};

/// Main menu actions.
#[derive(Clone, Debug)]
pub enum Action {
    FileOpenPDF,
    FileQuit,
    EditUndo,
    EditRedo,
    TemplateNew,
    TemplateOpen,
    TemplateSave,
    TemplateImportDir,
    TemplateImportZip,
    TemplateExportZip,
    HelpAbout,
}

new_action_group!(MenuActionGroup, "main-menu");
new_stateless_action!(FileOpenPDF, MenuActionGroup, "file_open_pdf");
new_stateless_action!(FileQuit, MenuActionGroup, "file_quit");
new_stateless_action!(EditUndo, MenuActionGroup, "edit_undo");
new_stateless_action!(EditRedo, MenuActionGroup, "edit_redo");
new_stateless_action!(TemplateNew, MenuActionGroup, "template_new");
new_stateless_action!(TemplateOpen, MenuActionGroup, "template_open");
new_stateless_action!(TemplateSave, MenuActionGroup, "template_save");
new_stateless_action!(TemplateImportZip, MenuActionGroup, "template_import_zip");
new_stateless_action!(TemplateImportDir, MenuActionGroup, "template_import_dir");
new_stateless_action!(TemplateExportZip, MenuActionGroup, "template_export_zip");
new_stateless_action!(HelpAbout, MenuActionGroup, "help_about");

/// Installs the menu on [Application] startup.
pub fn install_on_startup(app: &Application) {
    menu! {
        main_menu: {
            "File" {
                section! {
                    "Open PDF..." => FileOpenPDF,
                },
                section! {
                    "Quit" => FileQuit,
                },
            },
            "Edit" {
                "Undo" => EditUndo,
                "Redo" => EditRedo,
            },
            "Extraction template" {
                section! {
                    "New" => TemplateNew,
                    "Open..." => TemplateOpen,
                    "Save" => TemplateSave,
                },
                section! {
                    "Import from folder..." => TemplateImportDir,
                    "Import from ZIP..." => TemplateImportZip,
                },
                section! {
                    "Export to ZIP..." => TemplateExportZip,
                },
            },
            "Help" {
                "About..." => HelpAbout,
            },
        }
    }

    app.connect_startup(move |gtk_app| {
        gtk_app.set_menubar(Some(&main_menu));
    });
}

/// Attaches the menu's [RelmActionGroup] to the given [gtk::Widget], and calls the `callback` with
/// actions chosen in the menu.
pub fn init_for_widget<Callback, Widget>(widget: Widget, callback: Callback)
where
    Callback: Fn(Action) + 'static,
    Callback: Clone,
    Widget: AsRef<gtk::Widget>,
{
    let mut r = ActionRegistrator::new(callback);

    r.stateless(FileOpenPDF, Action::FileOpenPDF);
    r.stateless(FileQuit, Action::FileQuit);
    r.stateless(EditUndo, Action::EditUndo);
    r.stateless(EditRedo, Action::EditRedo);
    r.stateless(TemplateNew, Action::TemplateNew);
    r.stateless(TemplateOpen, Action::TemplateOpen);
    r.stateless(TemplateSave, Action::TemplateSave);
    r.stateless(TemplateImportDir, Action::TemplateImportDir);
    r.stateless(TemplateImportZip, Action::TemplateImportZip);
    r.stateless(TemplateExportZip, Action::TemplateExportZip);
    r.stateless(HelpAbout, Action::HelpAbout);

    r.register_for_widget(widget);
}

struct ActionRegistrator<Callback> {
    group: RelmActionGroup<MenuActionGroup>,
    callback: Callback,
}

impl<Callback> ActionRegistrator<Callback>
where
    Callback: Fn(Action) + 'static,
    Callback: Clone,
{
    fn new(callback: Callback) -> Self {
        let group = RelmActionGroup::<MenuActionGroup>::new();
        Self { group, callback }
    }

    fn stateless<Name>(&mut self, _name: Name, output: Action)
    where
        Name: ActionName,
        Name::State: EmptyType,
        Name::Target: EmptyType,
    {
        let callback = self.callback.clone();
        let action: RelmAction<Name> = RelmAction::new_stateless(move |_| {
            callback(output.clone());
        });
        self.group.add_action(action);
    }

    fn register_for_widget<Widget>(self, widget: Widget)
    where
        Widget: AsRef<gtk::Widget>,
    {
        self.group.register_for_widget(widget);
    }
}
