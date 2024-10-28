use std::{cell::RefCell, sync::Arc};

use crate::{
    consts::TABSTER_ATTRIBUTE_NAME,
    dom_api::DOM,
    instance::update_tabster_by_attribute,
    tabster::TabsterCore,
    types::DOMAPI,
    utils::{create_element_tree_walker, NodeFilterEnum},
};
use web_sys::{
    js_sys::Set,
    wasm_bindgen::{JsCast, UnwrapThrowExt},
    Document, MutationObserverInit, MutationRecord, Node,
};

pub fn observe_mutations(doc: &Document, tabster: Arc<RefCell<TabsterCore>>) -> Box<dyn Fn()> {
    let on_mutation = {
        let doc = doc.clone();

        move |mutations: Vec<MutationRecord>, _| {
            let removed_nodes = Set::default();

            for mutation in mutations.into_iter() {
                let target = mutation.target();
                let removed = mutation.removed_nodes();
                let added = mutation.added_nodes();

                if mutation.type_() == "attributes" {
                    if mutation.attribute_name() == Some(TABSTER_ATTRIBUTE_NAME.to_string()) {
                        let target = target.unwrap_throw();
                        // removedNodes helps to make sure we are not recreating things
                        // for the removed elements.
                        // For some reason, if we do removeChild() and setAttribute() on the
                        // removed child in the same tick, both the child removal and the attribute
                        // change will be present in the mutation records. And the attribute change
                        // will follow the child removal.
                        // So, we remember the removed nodes and ignore attribute changes for them.
                        if !removed_nodes.has(&target) {
                            update_tabster_by_attribute(
                                tabster.clone(),
                                target.dyn_into().unwrap_throw(),
                                None,
                            );
                        }
                    }
                } else {
                    removed.length();
                    for i in 0..removed.length() {
                        let removed_node = removed.item(i).unwrap_throw();
                        removed_nodes.add(&removed_node);
                        update_tabster_elements(&doc, removed_node, Some(true));
                        // tabster._dummyObserver.domChanged?.(target as HTMLElement);
                    }

                    for i in 0..added.length() {
                        let added_node = added.item(i).unwrap_throw();
                        update_tabster_elements(&doc, added_node, None);
                        // tabster._dummyObserver.domChanged?.(target as HTMLElement);
                    }
                }
            }

            removed_nodes.clear();
        }
    };

    let observer = DOM::create_mutation_observer(on_mutation);
    let init = MutationObserverInit::new();
    init.set_child_list(true);
    init.set_subtree(true);
    init.set_attributes(true);
    let val = serde_wasm_bindgen::to_value(&vec![TABSTER_ATTRIBUTE_NAME]).unwrap_throw();
    init.set_attribute_filter(&val);

    observer.observe_with_options(&doc, &init).unwrap_throw();

    Box::new(move || {
        observer.disconnect();
    })
}

fn update_tabster_elements(doc: &Document, node: Node, removed: Option<bool>) {
    // if (!elementByUId) {
    //     elementByUId = getInstanceContext(getWindow).elementByUId;
    // }

    process_node(node.clone(), removed);

    let walker = create_element_tree_walker(doc, &node, move |element| {
        return process_node(element, removed);
    });

    if let Some(walker) = walker {
        while walker.next_node().unwrap_throw().is_some() {
            /* Iterating for the sake of calling processNode() callback. */
        }
    }
}

fn process_node(element: Node, removed: Option<bool>) -> u32 {
    if element.node_type() == Node::TEXT_NODE {
        // It might actually be a text node.
        return *NodeFilterEnum::FilterSkip;
    }

    // const uid = (element as HTMLElementWithUID).__tabsterElementUID;

    // if (uid && elementByUId) {
    if removed.unwrap_or_default() {
        //         delete elementByUId[uid];
    } else {
        //         elementByUId[uid] ??= new WeakHTMLElement(getWindow, element);
    }
    // }

    // if (
    //     getTabsterOnElement(tabster, element) ||
    //     element.hasAttribute(TABSTER_ATTRIBUTE_NAME)
    // ) {
    //     updateTabsterByAttribute(tabster, element, removed);
    // }

    *NodeFilterEnum::FilterSkip
}
