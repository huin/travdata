use gtk::{gio, glib};

use crate::template;

glib::wrapper! {
    pub struct GroupGObject(ObjectSubclass<imp::GroupGObject>);
}

impl GroupGObject {
    pub fn new(name: &str) -> Self {
        glib::Object::builder().property("name", name).build()
    }

    pub fn root_list_store_from_group(group: &template::Group) -> gio::ListStore {
        group
            .groups
            .iter()
            .map(|(name, group)| {
                let group_list_store = Self::root_list_store_from_group(group);
                GroupGObject::new_with_groups(name, group_list_store)
            })
            .collect()
    }

    fn new_with_groups(name: &str, groups: gio::ListStore) -> Self {
        glib::Object::builder()
            .property("name", name)
            .property("groups", groups)
            .build()
    }
}

mod imp {
    use std::cell::RefCell;

    use gtk::{gio, glib, prelude::*, subclass::prelude::*};

    #[derive(glib::Properties)]
    #[properties(wrapper_type = super::GroupGObject)]
    pub struct GroupGObject {
        #[property(get, set)]
        name: RefCell<String>,
        #[property(get, set)]
        groups: RefCell<gio::ListStore>,
        // TODO: Add tables.
    }

    impl Default for GroupGObject {
        fn default() -> Self {
            Self {
                name: Default::default(),
                groups: RefCell::new(gio::ListStore::new::<super::GroupGObject>()),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GroupGObject {
        const NAME: &'static str = "GroupGObject";
        type Type = super::GroupGObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for GroupGObject {}
}
