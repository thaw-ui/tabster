use std::{cell::RefCell, sync::Arc};

use crate::{
    console_log,
    consts::TABSTER_ATTRIBUTE_NAME,
    dom_api::DOM,
    instance::{get_tabster_on_element, update_tabster_by_attribute},
    tabster::TabsterCore,
    types::{GetWindow, DOMAPI},
    utils::{create_element_tree_walker, get_instance_context, NodeFilterEnum},
};
use web_sys::{
    js_sys::Set,
    wasm_bindgen::{JsCast, UnwrapThrowExt},
    Document, Element, HtmlElement, MutationObserverInit, MutationRecord, Node,
};

pub fn observe_mutations(
    doc: &Document,
    tabster: Arc<RefCell<TabsterCore>>,
    sync_state: bool,
) -> Box<dyn Fn()> {
    console_log!("fn observe_mutations");

    let get_window = {
        let tabster = tabster.borrow();
        tabster.get_window.clone()
    };
    let element_by_uid = Arc::new(RefCell::new(None::<String>));

    let on_mutation = {
        let doc = doc.clone();
        let tabster = tabster.clone();
        let get_window = get_window.clone();
        let element_by_uid = element_by_uid.clone();

        move |mutations: Vec<MutationRecord>, _| {
            console_log!("fn observe_mutations on_mutation");
            let removed_nodes = Set::default();

            for mutation in mutations.into_iter() {
                let target = mutation.target();
                let removed = mutation.removed_nodes();
                let added = mutation.added_nodes();

                if mutation.type_() == "attributes" {
                    console_log!(
                        "fn observe_mutations on_mutation attributes {:#?}",
                        mutation.attribute_name()
                    );
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
                                &tabster,
                                &target.dyn_into().unwrap_throw(),
                                None,
                            );
                        }
                    }
                } else {
                    removed.length();
                    for i in 0..removed.length() {
                        let removed_node = removed.item(i).unwrap_throw();
                        removed_nodes.add(&removed_node);
                        update_tabster_elements(
                            tabster.clone(),
                            &get_window,
                            &doc,
                            element_by_uid.clone(),
                            removed_node,
                            Some(true),
                        );
                        // tabster._dummyObserver.domChanged?.(target as HTMLElement);
                    }

                    for i in 0..added.length() {
                        let added_node = added.item(i).unwrap_throw();
                        update_tabster_elements(
                            tabster.clone(),
                            &get_window,
                            &doc,
                            element_by_uid.clone(),
                            added_node,
                            None,
                        );
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

    if sync_state {
        let doc = get_window().document().unwrap_throw();
        update_tabster_elements(
            tabster,
            &get_window,
            &doc,
            element_by_uid,
            doc.body().unwrap_throw().into(),
            None,
        );
    }

    Box::new(move || {
        observer.disconnect();
    })
}

fn update_tabster_elements(
    tabster: Arc<RefCell<TabsterCore>>,
    get_window: &Arc<GetWindow>,
    doc: &Document,
    element_by_uid: Arc<RefCell<Option<String>>>,
    node: Node,
    removed: Option<bool>,
) {
    {
        let element_by_uid = element_by_uid.borrow_mut();
        if element_by_uid.is_none() {
            let context = get_instance_context(get_window);
            // element_by_uid = context.element_by_uid;
        }
    }
    // if (!elementByUId) {
    //     elementByUId = getInstanceContext(getWindow).elementByUId;
    // }

    process_node(&tabster, node.clone().dyn_into().unwrap_throw(), removed);

    let walker = create_element_tree_walker(doc, &node, move |element| {
        return process_node(&tabster, element.dyn_into().unwrap_throw(), removed);
    });

    if let Some(walker) = walker {
        while walker.next_node().unwrap_throw().is_some() {
            /* Iterating for the sake of calling processNode() callback. */
        }
    }
}

fn process_node(
    tabster: &Arc<RefCell<TabsterCore>>,
    element: Element,
    removed: Option<bool>,
) -> u32 {
    if element.node_type() == Node::TEXT_NODE {
        // It might actually be a text node.
        return *NodeFilterEnum::FilterSkip;
    }

    // TODO new code
    if !element.is_instance_of::<HtmlElement>() {
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

    let element: HtmlElement = element.dyn_into().unwrap_throw();
    if get_tabster_on_element(&tabster, &element).is_some()
        || element.has_attribute(TABSTER_ATTRIBUTE_NAME)
    {
        update_tabster_by_attribute(&tabster, &element, removed);
    }

    *NodeFilterEnum::FilterSkip
}
