use crate::{consts::TABSTER_ATTRIBUTE_NAME, tabster::TabsterCore, types::{self, TabsterAttributeOnElement, TabsterAttributeProps}};
use std::{cell::RefCell, sync::Arc};
use web_sys::{wasm_bindgen::UnwrapThrowExt, HtmlElement};

pub fn get_tabster_on_element(
    tabster: Arc<RefCell<TabsterCore>>,
    element: &HtmlElement,
) -> Option<Arc<types::TabsterOnElement>> {
    let mut tabster = tabster.borrow_mut();
    let entry = tabster.storage_entry(&element, None)?;
    let entry = entry.borrow();
    entry.tabster.clone()
}


pub fn update_tabster_by_attribute(mut tabster: TabsterCore,
    element: HtmlElement,
    dispose: Option<bool>) {
    let new_attr_value =
        if dispose.unwrap_or_default() || tabster.noop {
            None
        } else {
            element.get_attribute(TABSTER_ATTRIBUTE_NAME)
        };

    let mut entry = tabster.storage_entry(&element, None);
    let mut new_attr: Option<types::TabsterAttributeOnElement> = None;

    if let Some(new_attr_value) = new_attr_value {
        let Some(entry) = entry.as_ref() else {
            return;
        };

        let entry = entry.borrow();
        let Some(attr) = entry.attr.as_ref() else {
            return;
        };

        if new_attr_value == attr.string {
            return;
        }


        let new_value = match serde_json::from_str::<types::TabsterAttributeProps>(&new_attr_value) {
            Ok(new_value) => new_value,
            Err(err) => {
                web_sys::console::error_1(&web_sys::wasm_bindgen::JsValue::from_str(&err.to_string()));
                return;
            },
        };

        new_attr = Some(TabsterAttributeOnElement{
            string: new_attr_value,
            object: new_value,
        });
    } else if entry.is_none() {
        return;
    }

    let entry = if let Some(entry) = entry {
        entry
    } else {
        tabster.storage_entry(&element, Some(true)).unwrap_throw()
    };

    {
        let mut entry = entry.borrow_mut();
        if entry.tabster.is_none() {
            entry.tabster = Default::default();
        }
    }
    

    // let tabsterOnElement = entry.tabster || {};
    // const oldTabsterProps = entry.attr?.object || {};
    let new_tabster_props = new_attr.unwrap_throw().object;

    let TabsterAttributeProps { groupper } = new_tabster_props;
    if let Some(groupper) = groupper {

    }
    
}