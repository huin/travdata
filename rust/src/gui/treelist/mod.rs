use datamodel::item::TreeItemData;
use gtk::{
    gio, glib,
    prelude::{Cast, CastNone, GObjectPropertyExpressionExt, ListItemExt, ListModelExt, WidgetExt},
};
use relm4::{Component, ComponentParts, ComponentSender};

mod datamodel;

pub struct TreeList {}

impl Component for TreeList {
    type CommandOutput = ();
    type Input = ();
    type Output = ();
    type Init = ();
    type Root = gtk::ColumnView;
    type Widgets = gtk::ColumnView;

    fn init_root() -> Self::Root {
        gtk::ColumnView::default()
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        root.set_hexpand(true);
        root.set_vexpand(true);

        let root_model = gio::ListStore::new::<TreeItemData>();
        for i in 0..1000 {
            let item = TreeItemData::new(&format!("item-{}", i));
            root_model.append(&item);
            if i % 10 == 0 {
                for j in 0..3 {
                    item.children()
                        .append(&TreeItemData::new(&format!("item-{}-{}", i, j)));
                }
            }
        }
        let tree_model = gtk::TreeListModel::new(root_model, false, false, |item| {
            Some(
                item.downcast_ref::<TreeItemData>()
                    .expect("Needs to be `TreeItemData`")
                    .children()
                    .clone()
                    .into(),
            )
        });
        let selection_model = gtk::SingleSelection::new(Some(tree_model));
        root.set_model(Some(&selection_model));

        let expander_col = expander_column();
        root.append_column(&expander_col);

        let name_col = name_column();
        root.append_column(&name_col);

        root.connect_activate(move |column_view, position| {
            let model = column_view.model().expect("The model must exist");
            let row = model
                .item(position)
                .and_downcast::<gtk::TreeListRow>()
                .expect("Needs to be `gtk::TreeListRow`");
            let item_data = row
                .item()
                .and_downcast::<TreeItemData>()
                .expect("Needs to be `TreeItemData`");

            log::warn!("Activated item {:?}.", item_data);

            item_data
                .children()
                .append(&TreeItemData::new("created item"));
            item_data.notify_children();
        });

        let model = TreeList {};

        ComponentParts {
            model,
            widgets: root,
        }
    }
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
            .chain_property::<TreeItemData>("children")
            .chain_closure::<bool>(glib::closure!(
                |_: Option<glib::Object>, children: &gio::ListStore| { children.n_items() == 0 }
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
            .chain_property::<TreeItemData>("name")
            .bind(&label, "label", gtk::Widget::NONE);
    });
    gtk::ColumnViewColumn::builder()
        .title("")
        .factory(&factory)
        .build()
}

fn name_column() -> gtk::ColumnViewColumn {
    let factory = gtk::SignalListItemFactory::new();
    factory.connect_setup(move |_factory, list_item| {
        let label = gtk::Label::new(None);
        label.set_halign(gtk::Align::Start);
        let list_item = list_item
            .downcast_ref::<gtk::ListItem>()
            .expect("Needs to be `gtk::ListItem`");
        list_item.set_child(Some(&label));

        // Bind `list_item->item->name` to `label->label`.
        list_item
            .property_expression("item")
            .chain_property::<gtk::TreeListRow>("item")
            .chain_property::<TreeItemData>("name")
            .bind(&label, "label", gtk::Widget::NONE);
    });
    gtk::ColumnViewColumn::builder()
        .title("Name")
        .factory(&factory)
        .build()
}
