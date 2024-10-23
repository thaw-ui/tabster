use crate::{consts::TABSTER_ATTRIBUTE_NAME, dom_api::DOM, types::DOMAPI};
use web_sys::{
    js_sys::Set, wasm_bindgen::UnwrapThrowExt, Document, MutationObserverInit, MutationRecord,
};

pub fn observe_mutations(doc: &Document) -> Box<dyn Fn()> {
    let on_mutation = move |mutations: Vec<MutationRecord>, _| {
        let removed_nodes = Set::default();

        for mutation in mutations.into_iter() {
            let target = mutation.target();

            if mutation.type_() == "attributes" {
                if mutation.attribute_name() == Some(TABSTER_ATTRIBUTE_NAME.to_string()) {
                    // removedNodes helps to make sure we are not recreating things
                    // for the removed elements.
                    // For some reason, if we do removeChild() and setAttribute() on the
                    // removed child in the same tick, both the child removal and the attribute
                    // change will be present in the mutation records. And the attribute change
                    // will follow the child removal.
                    // So, we remember the removed nodes and ignore attribute changes for them.
                    if !removed_nodes.has(&target.unwrap_throw()) {
                        // updateTabsterByAttribute(
                        //     tabster,
                        //     target as HTMLElement
                        // );
                    }
                }
            } else {
            }
        }

        removed_nodes.clear();
    };

    let observer = DOM::create_mutation_observer(on_mutation);
    let init = MutationObserverInit::new();
    init.set_child_list(true);
    init.set_subtree(true);
    init.set_attributes(true);
    let val = serde_wasm_bindgen::to_value(&vec![TABSTER_ATTRIBUTE_NAME]).unwrap_throw();
    init.set_attribute_filter(&val);

    observer.observe_with_options(doc, &init).unwrap_throw();

    Box::new(move || {
        observer.disconnect();
    })
}
