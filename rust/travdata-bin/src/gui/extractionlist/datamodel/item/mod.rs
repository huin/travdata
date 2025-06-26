use gtk::glib;

mod imp;

glib::wrapper! {
    pub struct ItemData(ObjectSubclass<imp::ItemData>);
}

impl ItemData {
    pub fn new(name: &str) -> Self {
        glib::Object::builder().property("name", name).build()
    }
}
