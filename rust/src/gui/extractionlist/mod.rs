use datamodel::item::ItemData;
use gtk::{
    gio,
    prelude::{Cast, CastNone, GObjectPropertyExpressionExt, ListItemExt, ListModelExt, WidgetExt},
};
use relm4::{ComponentParts, ComponentSender, SimpleComponent};

mod datamodel;

pub struct ExtractionList {
    list_view: gtk::ListView,
}

#[relm4::component(pub)]
impl SimpleComponent for ExtractionList {
    type Init = ();
    type Input = ();
    type Output = ();

    view! {
        gtk::ScrolledWindow {
            container_add: &model.list_view,
        }
    }

    fn update(&mut self, _message: Self::Input, _sender: ComponentSender<Self>) {}

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

        let list_view = gtk::ListView::new(Some(selection_model), Some(factory));
        list_view.set_hexpand(true);
        list_view.set_vexpand(true);

        list_view.connect_activate(move |list_view, position| {
            let model = list_view.model().expect("The model must exist.");
            let item_data = model
                .item(position)
                .and_downcast::<ItemData>()
                .expect("The item has to be an `ItemData`.");

            log::warn!("Activated item {}.", item_data.name());
        });

        let model = Self { list_view };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
