use crate::types;
use web_sys::{
    wasm_bindgen::{prelude::Closure, JsCast, UnwrapThrowExt},
    Document, Element, MutationObserver, MutationRecord, Node, NodeFilter, TreeWalker,
};

pub struct DOM;

impl types::DOMAPI for DOM {
    fn create_mutation_observer(
        callback: impl Fn(Vec<MutationRecord>, MutationObserver) + 'static,
    ) -> MutationObserver {
        let cb = Box::new(callback) as Box<dyn Fn(Vec<MutationRecord>, MutationObserver)>;
        let cb = Closure::wrap(cb);
        MutationObserver::new(cb.as_ref().unchecked_ref()).unwrap_throw()
    }

    fn create_tree_walker(
        doc: &Document,
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

    fn get_first_element_child(element: Option<Element>) -> Option<Element> {
        if let Some(element) = element {
            element.first_element_child()
        } else {
            None
        }
    }

    fn get_last_element_child(element: Option<Element>) -> Option<Element> {
        let Some(element) = element else {
            return None;
        };
        element.last_element_child()
    }

    fn get_next_element_sibling(element: Option<Element>) -> Option<Element> {
        let Some(element) = element else {
            return None;
        };

        element.next_element_sibling()
    }

    fn get_previous_element_sibling(element: Option<Element>) -> Option<Element> {
        let Some(element) = element else {
            return None;
        };
        element.previous_element_sibling()
    }

    fn append_child(parent: Node, child: Node) -> Node {
        parent.append_child(&child).unwrap_throw()
    }

    fn insert_before(parent: Node, child: Node, reference_child: Option<Node>) -> Node {
        parent
            .insert_before(&child, reference_child.as_ref())
            .unwrap_throw()
    }
}
