use std::cell::RefCell;

use gtk::{glib, prelude::*, subclass::prelude::*};

#[derive(Default, glib::Properties)]
#[properties(wrapper_type = super::ItemData)]
pub struct ItemData {
    #[property(get, set)]
    name: RefCell<String>,
}

#[glib::object_subclass]
impl ObjectSubclass for ItemData {
    const NAME: &'static str = "ItemData";
    type Type = super::ItemData;
}

#[glib::derived_properties]
impl ObjectImpl for ItemData {}
