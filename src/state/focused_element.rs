use web_sys::HtmlElement;

use crate::{tabster::TabsterCore, types};
use std::{cell::RefCell, sync::Arc};

pub struct FocusedElementState {
    tabster: Arc<RefCell<TabsterCore>>,
    win: Arc<types::GetWindow>,
}

impl FocusedElementState {
    pub fn new(tabster: Arc<RefCell<TabsterCore>>, get_window: Arc<types::GetWindow>) -> Self {
        Self {
            tabster,
            win: get_window,
        }
    }

    pub fn get_focused_element(&self) -> Option<HtmlElement> {
        // TODO
        None
    }
}
