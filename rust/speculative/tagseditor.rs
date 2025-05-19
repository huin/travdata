use gtk::{SignalListItemFactory, gio, glib, prelude::*};
use relm4::{binding::ConnectBinding, prelude::*};

/// Component for editing a set of tags.
pub struct TagsEditor {
    tags_string_list: gtk::StringList,
}

#[derive(Debug)]
pub enum Input {}

#[derive(Debug)]
pub enum Output {}

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

                #[name = "new_tag_entry"]
                gtk::Entry {
                    set_halign: gtk::Align::Start,
                    set_hexpand: true,
                },

                #[name = "add_button"]
                gtk::Button::from_icon_name("list-add-symbolic") {
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // TODO empty list initially
        let tags_string_list = gtk::StringList::new(&["foo", "bar", "baz"]);
        let selection_model = gtk::SingleSelection::new(Some(tags_string_list.clone()));

        let model = TagsEditor {
            tags_string_list: tags_string_list.clone(),
        };

        let widgets = view_output!();

        widgets
            .tag_list
            .bind_model(Some(&tags_string_list), |object| {
                let label = gtk::Label::new(None);
                let string_object = object
                    .downcast_ref::<gtk::StringObject>()
                    .expect("expected a GtkStringObject");
                string_object
                    .bind_property("string", &label, "label")
                    .sync_create()
                    .build();
                label.into()
            });

        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_, list_item| {
            let label = gtk::Label::new(None);
            let list_item = list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("must be ListItem");
            list_item.set_child(Some(&label));

            // Bind list_item->item->string to label->label.
            list_item
                .property_expression("item")
                .chain_property::<gtk::StringObject>("string")
                .bind(&label, "label", gtk::Widget::NONE);
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {}
    }
}
