use crate::{
    root::RootAPI,
    tabster::TabsterCore,
    types::{
        FindAllProps, FindFirstProps, FindFocusableOutputProps, FindFocusableProps,
        FocusableAcceptElementState,
    },
    utils::{create_element_tree_walker, get_last_child, NodeFilterEnum},
};
use std::{cell::RefCell, sync::Arc};
use web_sys::{
    wasm_bindgen::{JsCast, UnwrapThrowExt},
    HtmlElement,
};

pub struct FocusableAPI {
    tabster: TabsterCore,
}

impl FocusableAPI {
    pub fn new(tabster: TabsterCore) -> Self {
        Self { tabster }
    }

    pub fn find_last(&mut self, options: FindFirstProps, out: FindFocusableOutputProps) {
        self.find_element(
            FindFocusableProps {
                is_backward: Some(true),
                ..options.into()
            },
            out,
        );
    }

    pub fn find_all(&mut self, options: FindAllProps, out: FindFocusableOutputProps) {
        self.find_elements(true, options.into(), out);
    }

    fn find_element(
        &mut self,
        options: FindFocusableProps,
        out: FindFocusableOutputProps,
    ) -> Option<HtmlElement> {
        let found = self.find_elements(false, options, out);
        found.map(|found| found[0].clone())
    }

    fn find_elements(
        &mut self,
        is_find_all: bool,
        options: FindFocusableProps,
        mut out: FindFocusableOutputProps,
    ) -> Option<Vec<HtmlElement>> {
        let FindFocusableProps {
            container,
            current_element,
            is_backward,
            on_element,
        } = options;

        let mut elements = Vec::<HtmlElement>::new();

        let accept_element_state = FocusableAcceptElementState {
            container: container.clone(),
            from: current_element.unwrap_or_else(|| container.clone()),
            from_ctx: None,
            found: None,
            found_element: None,
            found_backward: None,
            skipped_focusable: None,
        };
        let accept_element_state = Arc::new(RefCell::new(accept_element_state));
        let Some(walker) =
            create_element_tree_walker(container.owner_document().unwrap_throw(), &container, {
                let accept_element_state = accept_element_state.clone();
                move |node| {
                    Self::accept_element(node.dyn_into().unwrap_throw(), &accept_element_state)
                }
            })
        else {
            return None;
        };

        let prepare_for_next_element = {
            let accept_element_state = accept_element_state.clone();
            move |should_continue_if_not_found: Option<bool>,
                  elements: &mut Vec<HtmlElement>,
                  on_element: &Option<Box<dyn Fn(HtmlElement) -> bool>>,
                  out: &mut FindFocusableOutputProps,
                  tabster: &mut TabsterCore|
                  -> bool {
                let mut accept_element_state = accept_element_state.try_borrow_mut().unwrap_throw();
                let found_element = if let Some(found_element) =
                    accept_element_state.found_element.clone()
                {
                    Some(found_element)
                } else if let Some(found_backward) = accept_element_state.found_backward.clone() {
                    Some(found_backward)
                } else {
                    None
                };

                if let Some(found_element) = found_element.clone() {
                    elements.push(found_element);
                }

                let found_element_state = found_element.is_some();
                if is_find_all {
                    if let Some(found_element) = found_element {
                        accept_element_state.found = Some(false);
                        accept_element_state.found_element = None;
                        accept_element_state.found_backward = None;
                        accept_element_state.from_ctx = None;
                        accept_element_state.from = found_element.clone();

                        if let Some(on_element) = on_element {
                            if !on_element(found_element) {
                                return false;
                            }
                        }
                    }
                    found_element_state || should_continue_if_not_found.unwrap_or_default()
                } else {
                    if let Some(found_element) = found_element {
                        out.uncontrolled = RootAPI::get_tabster_context(
                            tabster,
                            found_element.into(),
                            Default::default(),
                        )
                        .and_then(|ctx| ctx.uncontrolled);
                    }

                    should_continue_if_not_found.unwrap_or_default() && !found_element_state
                }
            }
        };

        if matches!(is_backward, Some(true)) {
            let Some(last_child) = get_last_child(container) else {
                return None;
            };
            if Self::accept_element(last_child.clone(), &accept_element_state)
                == *NodeFilterEnum::FilterAccept
                && !prepare_for_next_element(
                    Some(true),
                    &mut elements,
                    &on_element,
                    &mut out,
                    &mut self.tabster,
                )
            {
                let accept_element_state = accept_element_state.try_borrow().unwrap_throw();
                if matches!(accept_element_state.skipped_focusable, Some(true)) {
                    out.out_of_dom_order = Some(true);
                }

                return Some(elements);
            }

            walker.set_current_node(&last_child);
        }
        loop {
            if matches!(is_backward, Some(true)) {
                walker.previous_node().unwrap_throw();
            } else {
                walker.next_node().unwrap_throw();
            }

            if !prepare_for_next_element(
                None,
                &mut elements,
                &on_element,
                &mut out,
                &mut self.tabster,
            ) {
                break;
            }
        }

        let accept_element_state = accept_element_state.try_borrow().unwrap_throw();
        if matches!(accept_element_state.skipped_focusable, Some(true)) {
            out.out_of_dom_order = Some(true);
        }

        if elements.is_empty() {
            None
        } else {
            Some(elements)
        }
    }

    fn accept_element(
        element: HtmlElement,
        state: &Arc<RefCell<FocusableAcceptElementState>>,
    ) -> u32 {
        let state = state.try_borrow().unwrap_throw();
        if matches!(state.found, Some(true)) {
            return *NodeFilterEnum::FilterAccept;
        }
        todo!()
    }
}
