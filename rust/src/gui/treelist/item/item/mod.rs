use gtk::glib;

mod imp;

glib::wrapper! {
    pub struct TreeItemData(ObjectSubclass<imp::TreeItemData>);
}

impl TreeItemData {
    pub fn new(name: &str) -> Self {
        glib::Object::builder().property("name", name).build()
    }
}
