use crate::types;
use web_sys::{
    wasm_bindgen::{JsCast, UnwrapThrowExt},
    Document, Element, Node, NodeFilter, TreeWalker,
};

pub struct DOM;

impl types::DOMAPI for DOM {
    fn create_tree_walker(
        doc: Document,
        root: &Node,
        what_to_show: u32,
        filter: Option<&NodeFilter>,
    ) -> TreeWalker {
        doc.create_tree_walker_with_what_to_show_and_filter(root, what_to_show, filter)
            .unwrap_throw()
    }

    fn get_parent_node(node: Option<Node>) -> Option<types::ParentNode> {
        node?.parent_node()
    }

    fn get_parent_element(element: Option<web_sys::HtmlElement>) -> Option<web_sys::HtmlElement> {
        element?
            .parent_element()
            .map(|e| e.dyn_into().unwrap_throw())
    }

    fn node_contains(parent: Option<Node>, child: Option<Node>) -> bool {
        let Some(parent) = parent else {
            return false;
        };

        parent.contains(child.as_ref())
    }

    fn get_last_element_child(element: Option<Element>) -> Option<Element> {
        let Some(element) = element else {
            return None;
        };
        element.last_element_child()
    }
}
