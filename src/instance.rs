use crate::{tabster::TabsterCore, types};
use std::{cell::RefCell, sync::Arc};
use web_sys::HtmlElement;

pub fn get_tabster_on_element(
    tabster: Arc<RefCell<TabsterCore>>,
    element: &HtmlElement,
) -> Option<Arc<types::TabsterOnElement>> {
    let mut tabster = tabster.borrow_mut();
    let entry = tabster.storage_entry(&element, None)?;
    entry.tabster.clone()
}
