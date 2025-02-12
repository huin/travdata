use gtk::{
    gio, glib,
    prelude::{Cast, CastNone, GObjectPropertyExpressionExt, ListItemExt, ListModelExt, WidgetExt},
};
use relm4::{Component, ComponentParts, ComponentSender};

use crate::{
    config::book,
    gui::{gobjects::group::GroupGObject, util},
};

/// Initialisation data for [GroupTreeList].
pub struct Init {
    pub yaml_group: Option<book::YamlGroup>,
}

/// Input messages for [GroupTreeList].
#[derive(Debug)]
pub enum Input {
    LoadRootGroup(Option<book::YamlGroup>),
}

/// Output messages for [GroupTreeList].
#[derive(Debug)]
pub enum Output {
    ActivatedRow(GroupGObject),
}

/// Component to view a hierarchy of [book::YamlGroup].
///
/// It emits [GroupGObject] values that are copies of the original [book::YamlGroup].
pub struct GroupTreeList {}

impl Component for GroupTreeList {
    type CommandOutput = ();
    type Input = Input;
    type Output = Output;
    type Init = Init;
    type Root = gtk::ColumnView;
    type Widgets = gtk::ColumnView;

    fn init_root() -> Self::Root {
        gtk::ColumnView::default()
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        root.set_hexpand(true);
        root.set_vexpand(true);

        let selection_model = create_model(init.yaml_group);
        root.set_model(Some(&selection_model));

        let expander_col = expander_column();
        root.append_column(&expander_col);

        root.connect_activate(move |column_view, position| {
            let group_gobject = column_view
                .model()
                .expect("The model must exist")
                .item(position)
                .and_downcast::<gtk::TreeListRow>()
                .expect("Needs to be `gtk::TreeListRow`")
                .item()
                .and_downcast::<GroupGObject>()
                .expect("Needs to be `GroupGObject`");

            util::send_output_or_log(
                Output::ActivatedRow(group_gobject),
                "activated row",
                &sender,
            );
        });

        let model = GroupTreeList {};

        ComponentParts {
            model,
            widgets: root,
        }
    }

    fn builder() -> relm4::ComponentBuilder<Self> {
        relm4::ComponentBuilder::<Self>::default()
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, root: &Self::Root) {
        match message {
            Input::LoadRootGroup(yaml_group) => {
                let selection_model = create_model(yaml_group);
                root.set_model(Some(&selection_model));
            }
        }
    }
}

fn create_model(yaml_group: Option<book::YamlGroup>) -> gtk::SingleSelection {
    let root_model = match yaml_group {
        Some(yaml_group) => GroupGObject::root_list_store_from_yaml_group(&yaml_group),
        None => gio::ListStore::new::<GroupGObject>(),
    };
    let tree_model = gtk::TreeListModel::new(root_model, false, false, |item| {
        Some(
            item.downcast_ref::<GroupGObject>()
                .expect("Needs to be `GroupGObject`")
                .groups()
                .clone()
                .into(),
        )
    });
    gtk::SingleSelection::new(Some(tree_model))
}

fn expander_column() -> gtk::ColumnViewColumn {
    let factory = gtk::SignalListItemFactory::new();
    factory.connect_setup(move |_factory, list_item| {
        let expander = gtk::TreeExpander::new();
        let list_item = list_item
            .downcast_ref::<gtk::ListItem>()
            .expect("Needs to be `gtk::ListItem`");
        list_item.set_child(Some(&expander));

        list_item
            .property_expression("item")
            .chain_property::<gtk::TreeListRow>("item")
            .chain_property::<GroupGObject>("groups")
            .chain_closure::<bool>(glib::closure!(
                |_: Option<glib::Object>, groups: &gio::ListStore| { groups.n_items() == 0 }
            ))
            .bind(&expander, "hide-expander", glib::Object::NONE);
        list_item
            .property_expression("item")
            .bind(&expander, "list-row", glib::Object::NONE);

        let label = gtk::Label::new(None);
        label.set_halign(gtk::Align::Start);
        expander.set_child(Some(&label));

        // Bind `list_item->item->name` to `label->label`.
        list_item
            .property_expression("item")
            .chain_property::<gtk::TreeListRow>("item")
            .chain_property::<GroupGObject>("name")
            .bind(&label, "label", gtk::Widget::NONE);
    });
    gtk::ColumnViewColumn::builder()
        .title("")
        .factory(&factory)
        .build()
}
