use std::cell::RefCell;

use gtk::{gio, glib, prelude::*, subclass::prelude::*};

#[derive(glib::Properties)]
#[properties(wrapper_type = super::TreeItemData)]
pub struct TreeItemData {
    #[property(get, set)]
    name: RefCell<String>,
    #[property(get, set)]
    children: RefCell<gio::ListStore>,
}

impl Default for TreeItemData {
    fn default() -> Self {
        Self {
            name: Default::default(),
            children: RefCell::new(gio::ListStore::new::<super::TreeItemData>()),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for TreeItemData {
    const NAME: &'static str = "TreeItemData";
    type Type = super::TreeItemData;
}

#[glib::derived_properties]
impl ObjectImpl for TreeItemData {}
