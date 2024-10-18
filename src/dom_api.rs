use crate::types;
use web_sys::{wasm_bindgen::UnwrapThrowExt, Document, Element, Node, NodeFilter, TreeWalker};

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

    fn get_last_element_child(element: Option<Element>) -> Option<Element> {
        let Some(element) = element else {
            return None;
        };
        element.last_element_child()
    }
}
