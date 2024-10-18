use crate::{tabster::TabsterCore, types};
use web_sys::HtmlElement;

pub fn get_tabster_on_element(
    tabster: TabsterCore,
    element: HtmlElement,
) -> Option<types::TabsterOnElement> {
    return tabster.storage_entry(element, None)?.tabster;
}
