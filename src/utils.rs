use crate::{dom_api::Dom, types::DOMAPI};
use std::ops::Deref;
use web_sys::{
    wasm_bindgen::{prelude::Closure, JsCast},
    Document, Node, NodeFilter, TreeWalker,
};

pub fn create_element_tree_walker(
    doc: Document,
    root: &Node,
    accept_node: impl Fn(Node) -> u32 + 'static,
) -> Option<TreeWalker> {
    if root.node_type() != Node::ELEMENT_NODE {
        return None;
    }
    let node_filter = NodeFilter::new();
    let cb: Closure<dyn Fn(Node) -> u32> = Closure::new(accept_node);
    node_filter.set_accept_node(cb.as_ref().unchecked_ref());

    Some(Dom::create_tree_walker(
        doc,
        root,
        *NodeFilterEnum::ShowElement,
        Some(&node_filter),
    ))
}

pub enum NodeFilterEnum {
    FilterAccept,
    ShowElement,
}

impl Deref for NodeFilterEnum {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::FilterAccept => &1,
            Self::ShowElement => &0x1,
        }
    }
}
