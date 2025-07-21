use datamodel::item::ItemData;
use gtk::{
    gio,
    prelude::{Cast, CastNone, GObjectPropertyExpressionExt, ListItemExt, ListModelExt, WidgetExt},
};
use relm4::{Component, ComponentParts, ComponentSender};

mod datamodel;

pub struct ExtractionList {}

impl Component for ExtractionList {
    type CommandOutput = ();
    type Input = ();
    type Output = ();
    type Init = ();
    type Root = gtk::ListView;
    type Widgets = gtk::ListView;

    fn init_root() -> Self::Root {
        gtk::ListView::default()
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let list_model = gio::ListStore::new::<ItemData>();
        for i in 0..1000 {
            list_model.append(&ItemData::new(&format!("item-{}", i)));
        }

        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(move |_factory, list_item| {
            let row = gtk::Label::new(None);
            let list_item = list_item
                .downcast_ref::<gtk::ListItem>()
                .expect("Needs to be ListItem");
            list_item.set_child(Some(&row));

            // Bind `list_item->item->name` to `label->label`.
            list_item
                .property_expression("item")
                .chain_property::<ItemData>("name")
                .bind(&row, "label", gtk::Widget::NONE);
        });

        let selection_model = gtk::SingleSelection::new(Some(list_model.clone()));

        root.set_model(Some(&selection_model));
        root.set_factory(Some(&factory));

        root.set_hexpand(true);
        root.set_vexpand(true);

        root.connect_activate(move |list_view, position| {
            let model = list_view.model().expect("The model must exist.");
            let item_data = model
                .item(position)
                .and_downcast::<ItemData>()
                .expect("The item has to be an `ItemData`.");

            log::warn!("Activated item {}.", item_data.name());
        });

        let model = ExtractionList {};

        ComponentParts {
            model,
            widgets: root,
        }
    }
}
