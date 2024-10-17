use web_sys::{wasm_bindgen::UnwrapThrowExt, NodeFilter};

use crate::types::DOMAPI;

pub struct Dom;

impl DOMAPI for Dom {
    fn create_tree_walker(
        doc: web_sys::Document,
        root: &web_sys::Node,
        what_to_show: u32,
        filter: Option<&NodeFilter>,
    ) -> web_sys::TreeWalker {
        doc.create_tree_walker_with_what_to_show_and_filter(root, what_to_show, filter)
            .unwrap_throw()
    }
}
