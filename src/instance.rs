use crate::{
    console_error, console_log,
    consts::TABSTER_ATTRIBUTE_NAME,
    tabster::TabsterCore,
    types::{self, TabsterAttributeOnElement, TabsterAttributeProps, TabsterOnElement},
    web::console_log_node,
};
use std::{cell::RefCell, sync::Arc};
use web_sys::{wasm_bindgen::UnwrapThrowExt, HtmlElement, Node};

pub fn get_tabster_on_element(
    tabster: &Arc<RefCell<TabsterCore>>,
    element: &Node,
) -> Option<Arc<RefCell<types::TabsterOnElement>>> {
    let mut tabster = tabster.borrow_mut();
    let entry = tabster.storage_entry(&element, None)?;
    let entry = entry.borrow();
    entry.tabster.clone()
}

pub fn update_tabster_by_attribute(
    tabster: &Arc<RefCell<TabsterCore>>,
    element: &HtmlElement,
    dispose: Option<bool>,
) {
    console_log_node(&element);
    console_log!("fn update_tabster_by_attribute");
    let mut tabster_ref = tabster.borrow_mut();
    let new_attr_value = if dispose.unwrap_or_default() || tabster_ref.noop {
        None
    } else {
        element.get_attribute(TABSTER_ATTRIBUTE_NAME)
    };

    let entry = tabster_ref.storage_entry(&element, None);
    let mut new_attr: Option<types::TabsterAttributeOnElement> = None;

    if let Some(new_attr_value) = new_attr_value {
        if let Some(entry) = entry.as_ref() {
            let entry = entry.borrow();
            if let Some(attr) = entry.attr.as_ref() {
                if new_attr_value == attr.string {
                    return;
                }
            }
        }

        let new_value = match serde_json::from_str::<types::TabsterAttributeProps>(&new_attr_value)
        {
            Ok(new_value) => new_value,
            Err(err) => {
                web_sys::console::error_1(&web_sys::wasm_bindgen::JsValue::from_str(
                    &err.to_string(),
                ));
                return;
            }
        };

        new_attr = Some(TabsterAttributeOnElement {
            string: new_attr_value,
            object: Arc::new(new_value),
        });
    } else if entry.is_none() {
        return;
    }

    let entry = if let Some(entry) = entry {
        entry
    } else {
        tabster_ref
            .storage_entry(&element, Some(true))
            .unwrap_throw()
    };

    {
        let mut entry = entry.borrow_mut();
        if entry.tabster.is_none() {
            entry.tabster = Some(Default::default());
        }
    }
    let mut entry = entry.borrow_mut();
    let some_entry_tabster = Arc::new(RefCell::new(TabsterOnElement::default()));
    let tabster_on_element = entry.tabster.clone().unwrap_or(some_entry_tabster);
    // const oldTabsterProps = entry.attr?.object || {};
    let some_new_attr_object = Arc::new(TabsterAttributeProps::default());
    let new_tabster_props = if let Some(new_attr) = &new_attr {
        new_attr.object.clone()
    } else {
        some_new_attr_object
    };

    if let Some(new_tabster_props_groupper) = &new_tabster_props.groupper {
        let sys = new_tabster_props.sys.clone();
        let mut tabster_on_element = tabster_on_element.borrow_mut();
        if tabster_on_element.groupper.is_some() {
            // tabster_on_element.groupper.setProps(
            //     newTabsterProps.groupper as Types.GroupperProps
            // );
        } else {
            let groupper = tabster_ref.groupper.clone();
            drop(tabster_ref);
            if let Some(groupper) = groupper {
                let mut groupper = groupper.borrow_mut();
                tabster_on_element.groupper = Some(groupper.create_groupper(
                    &element,
                    new_tabster_props_groupper.clone(),
                    sys,
                ));
            } else if cfg!(debug_assertions) {
                console_error!(
                    "Groupper API used before initialization, please call `getGroupper()`"
                )
            }
        }
    } else if let Some(new_tabster_props_mover) = &new_tabster_props.mover {
        let sys = new_tabster_props.sys.clone();
        let mut tabster_on_element = tabster_on_element.borrow_mut();
        if tabster_on_element.mover.is_some() {
            // tabsterOnElement.mover.setProps(
            //     newTabsterProps.mover as Types.MoverProps
            // );
        } else {
            let mover = tabster_ref.mover.clone();
            drop(tabster_ref);
            if let Some(mover) = mover {
                let mut mover = mover.borrow_mut();
                tabster_on_element.mover =
                    Some(mover.create_mover(&element, new_tabster_props_mover.clone(), sys));
            } else if cfg!(debug_assertions) {
                console_error!("Mover API used before initialization, please call `getMover()`");
            }
        }
    }

    if let Some(new_attr) = new_attr {
        entry.attr = Some(new_attr);
    } else {
        let tabster_on_element = tabster_on_element.borrow();
        if tabster_on_element.is_empty() {
            entry.tabster = None;
            entry.attr = None;
        }
        let mut tabster_ref = tabster.borrow_mut();
        tabster_ref.storage_entry(&element, Some(false));
    }
}
