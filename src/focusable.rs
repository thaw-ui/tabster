use crate::{
    consts::FOCUSABLE_SELECTOR,
    dom_api::DOM,
    instance::get_tabster_on_element,
    root::RootAPI,
    tabster::TabsterCore,
    types::{
        self, FindAllProps, FindFirstProps, FindFocusableOutputProps, FindFocusableProps,
        FocusableAcceptElementState, DOMAPI,
    },
    utils::{
        create_element_tree_walker, get_dummy_input_container, get_last_child, is_display_none,
        matches_selector, should_ignore_focus, NodeFilterEnum,
    },
};
use std::{cell::RefCell, sync::Arc};
use web_sys::{
    wasm_bindgen::{JsCast, UnwrapThrowExt},
    Element, HtmlElement, Node, SvgElement,
};

#[derive(Clone)]
pub struct FocusableAPI {
    tabster: Arc<RefCell<TabsterCore>>,
}

impl FocusableAPI {
    pub fn new(tabster: Arc<RefCell<TabsterCore>>) -> Self {
        Self { tabster }
    }

    pub fn is_focusable(
        &self,
        el: &Element,
        include_programmatically_focusable: Option<bool>,
        no_visible_check: Option<bool>,
        no_accessible_check: Option<bool>,
    ) -> bool {
        fn tab_index(el: &Element) -> i32 {
            if let Ok(el) = el.clone().dyn_into::<HtmlElement>() {
                el.tab_index()
            } else if let Ok(el) = el.clone().dyn_into::<SvgElement>() {
                el.tab_index()
            } else {
                unreachable!()
            }
        }
        if matches_selector(&el, FOCUSABLE_SELECTOR)
            && (include_programmatically_focusable.unwrap_or_default() || tab_index(el) != -1)
        {
            (no_visible_check.unwrap_or_default() || FocusableAPI::is_visible(el))
                && (no_accessible_check.unwrap_or_default() || self.is_accessible(el))
        } else {
            false
        }
    }

    fn is_visible(el: &Element) -> bool {
        let Some(owner_document) = el.owner_document() else {
            return false;
        };
        if el.node_type() != Node::ELEMENT_NODE {
            return false;
        }

        if is_display_none(el.clone()) {
            return false;
        }

        let rect = owner_document
            .body()
            .unwrap_throw()
            .get_bounding_client_rect();

        if rect.width() == 0.0 && rect.height() == 0.0 {
            // This might happen, for example, if our <body> is in hidden <iframe>.
            return false;
        }

        true
    }

    fn is_accessible(&self, el: &Element) -> bool {
        let mut e = Some(el.clone());
        loop {
            let Some(e_ref) = e.as_ref() else {
                break;
            };

            let tabster_on_element = get_tabster_on_element(&self.tabster, e_ref);

            if self.is_hidden(e_ref) {
                return false;
            }
            let Some(tabster_on_element) = tabster_on_element else {
                return false;
            };
            let tabster_on_element = tabster_on_element.borrow();
            let Some(focusable) = tabster_on_element.focusable.as_ref() else {
                return false;
            };
            let ignore_disabled = focusable.ignore_aria_disabled;
            if !ignore_disabled.unwrap_or_default() && FocusableAPI::is_disabled(e_ref) {
                return false;
            }

            e = e.map(|e| e.parent_element()).flatten();
        }

        true
    }

    fn is_disabled(el: &Element) -> bool {
        el.has_attribute("disabled")
    }

    fn is_hidden(&self, el: &Element) -> bool {
        let Some(attr_val) = el.get_attribute("aria-hidden") else {
            return false;
        };

        if attr_val.to_lowercase() == "true" {
            let tabster = self.tabster.borrow();
            if let Some(modalizer) = tabster.modalizer.as_ref() {
                if !(modalizer.is_augmented)(el.clone()) {
                    return true;
                }
            }
        }

        false
    }

    pub fn find_first(
        &mut self,
        options: FindFirstProps,
        out: &mut FindFocusableOutputProps,
    ) -> Option<HtmlElement> {
        self.find_element(options.into(), out)
    }

    pub fn find_last(
        &mut self,
        options: FindFirstProps,
        out: &mut FindFocusableOutputProps,
    ) -> Option<HtmlElement> {
        self.find_element(
            FindFocusableProps {
                is_backward: Some(true),
                ..options.into()
            },
            out,
        )
    }

    pub fn find_next(
        &mut self,
        options: types::FindNextProps,
        out: &mut FindFocusableOutputProps,
    ) -> Option<HtmlElement> {
        self.find_element(options.into(), out)
    }

    pub fn find_prev(
        &mut self,
        options: types::FindNextProps,
        out: &mut FindFocusableOutputProps,
    ) -> Option<HtmlElement> {
        self.find_element(
            FindFocusableProps {
                is_backward: Some(true),
                ..options.into()
            },
            out,
        )
    }

    pub fn find_all(&mut self, options: FindAllProps, out: &mut FindFocusableOutputProps) {
        self.find_elements(true, options.into(), out);
    }

    fn find_element(
        &mut self,
        options: FindFocusableProps,
        out: &mut FindFocusableOutputProps,
    ) -> Option<HtmlElement> {
        let found = self.find_elements(false, options, out);
        found.map(|found| found[0].clone())
    }

    fn find_elements(
        &mut self,
        is_find_all: bool,
        options: FindFocusableProps,
        out: &mut FindFocusableOutputProps,
    ) -> Option<Vec<HtmlElement>> {
        let FindFocusableProps {
            container,
            current_element,
            include_programmatically_focusable,
            ignore_accessibility,
            use_active_modalizer,
            modalizer_id,
            is_backward,
            on_element,
            accept_condition,
            ..
        } = options;

        let mut elements = Vec::<HtmlElement>::new();

        let accept_condition = accept_condition.unwrap_or_else({
            || {
                let this = self.clone();
                Box::new(move |el| {
                    this.is_focusable(
                        &el,
                        include_programmatically_focusable,
                        Some(false),
                        ignore_accessibility,
                    )
                })
            }
        });

        let modalizer_user_id =
            if modalizer_id.is_none() && use_active_modalizer.unwrap_or_default() {
                let tabster = self.tabster.borrow();
                if let Some(modalizer) = tabster.modalizer.as_ref() {
                    modalizer.active_id.clone()
                } else {
                    None
                }
            } else {
                if let Some(modalizer_id) = modalizer_id {
                    Some(modalizer_id)
                } else {
                    let tabster_context =
                        RootAPI::get_tabster_context(&self.tabster, &container, Default::default());

                    tabster_context
                        .map(|c| c.modalizer.map(|m| m.user_id.clone()))
                        .flatten()
                }
            };

        let accept_element_state = FocusableAcceptElementState {
            modalizer_user_id,
            current_ctx: None,
            accept_condition,
            has_custom_condition: None,
            ignore_accessibility,
            container: container.clone(),
            from: current_element.clone().unwrap_or_else(|| container.clone()),
            from_ctx: None,
            found: None,
            found_element: None,
            found_backward: None,
            reject_elements_from: None,
            skipped_focusable: None,
        };
        let accept_element_state = Arc::new(RefCell::new(accept_element_state));
        let Some(walker) =
            create_element_tree_walker(&container.owner_document().unwrap_throw(), &container, {
                let accept_element_state = accept_element_state.clone();
                let this = self.clone();
                move |node| {
                    this.accept_element(node.dyn_into().unwrap_throw(), &accept_element_state)
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
                  tabster: Arc<RefCell<TabsterCore>>|
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
                            &tabster,
                            &found_element,
                            Default::default(),
                        )
                        .and_then(|ctx| ctx.uncontrolled);
                    }

                    should_continue_if_not_found.unwrap_or_default() && !found_element_state
                }
            }
        };

        if current_element.is_none() {
            out.out_of_dom_order = Some(true);
        }
        if current_element.is_some()
            && DOM::node_contains(
                Some(container.clone().into()),
                current_element.clone().map(|el| el.into()),
            )
        {
            walker.set_current_node(&current_element.clone().unwrap().into());
        } else if matches!(is_backward, Some(true)) {
            let Some(last_child) = get_last_child(container) else {
                return None;
            };
            if self.accept_element(last_child.clone().into(), &accept_element_state)
                == *NodeFilterEnum::FilterAccept
                && !prepare_for_next_element(
                    Some(true),
                    &mut elements,
                    &on_element,
                    out,
                    self.tabster.clone(),
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
                out,
                self.tabster.clone(),
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
        &self,
        element: Element,
        state: &Arc<RefCell<FocusableAcceptElementState>>,
    ) -> u32 {
        let mut state = state.try_borrow_mut().unwrap_throw();
        if matches!(state.found, Some(true)) {
            return *NodeFilterEnum::FilterAccept;
        }

        let found_backward = state.found_backward.clone();

        if found_backward.is_some()
            && (Some(&element) == found_backward.as_deref()
                || !DOM::node_contains(
                    found_backward.clone().map(|f| f.into()),
                    Some(element.clone().into()),
                ))
        {
            state.found = Some(true);
            state.found_element = found_backward;
            return *NodeFilterEnum::FilterAccept;
        }

        let container = state.container.clone();

        if element == *container {
            return *NodeFilterEnum::FilterSkip;
        }

        if !DOM::node_contains(Some(container.into()), Some(element.clone().into())) {
            return *NodeFilterEnum::FilterReject;
        }

        if get_dummy_input_container(&Some(element.clone())).is_some() {
            return *NodeFilterEnum::FilterReject;
        }

        if DOM::node_contains(
            state.reject_elements_from.clone().map(|r| r.into()),
            Some(element.clone().into()),
        ) {
            return *NodeFilterEnum::FilterReject;
        }

        state.current_ctx =
            RootAPI::get_tabster_context(&self.tabster, &element, Default::default());
        // Tabster is opt in, if it is not managed, don't try and get do anything special
        let Some(ctx) = state.current_ctx.clone() else {
            return *NodeFilterEnum::FilterSkip;
        };

        if should_ignore_focus(&element) {
            if self.is_focusable(
                &element.clone().dyn_into().unwrap_throw(),
                None,
                Some(true),
                Some(true),
            ) {
                state.skipped_focusable = Some(true);
            }

            return *NodeFilterEnum::FilterSkip;
        }

        // We assume iframes are focusable because native tab behaviour would tab inside.
        // But we do it only during the standard search when there is no custom accept
        // element condition.
        if !state.has_custom_condition.unwrap_or_default()
            && (element.tag_name() == "IFRAME" || element.tag_name() == "WEBVIEW")
        {
            let user_id = if let Some(modalizer) = &ctx.modalizer {
                Some(modalizer.user_id.clone())
            } else {
                None
            };
            let active_id = {
                let tabster = self.tabster.borrow();
                if let Some(modalizer) = &tabster.modalizer {
                    modalizer.active_id.clone()
                } else {
                    None
                }
            };

            if user_id == active_id {
                state.found = Some(true);
                let element: HtmlElement = element.clone().dyn_into().unwrap_throw();
                state.found_element = Some(element.clone());
                state.reject_elements_from = Some(element);

                return *NodeFilterEnum::FilterAccept;
            } else {
                return *NodeFilterEnum::FilterReject;
            }
        }

        if !state.ignore_accessibility.unwrap_or_default() && !self.is_accessible(&element) {
            if self.is_focusable(&element, Some(false), Some(true), Some(true)) {
                state.skipped_focusable = Some(true);
            }

            return *NodeFilterEnum::FilterReject;
        }

        let mut result = None::<u32>;

        let from_ctx = if let Some(from_ctx) = state.from_ctx.clone() {
            Some(from_ctx)
        } else {
            let from_ctx =
                RootAPI::get_tabster_context(&self.tabster, &state.from, Default::default());
            state.from_ctx = from_ctx.clone();
            from_ctx
        };

        let from_mover = from_ctx.clone().map(|c| c.mover).flatten();
        let groupper = ctx.groupper;
        let mover = ctx.mover;

        result = {
            let tabster = self.tabster.borrow();
            if let Some(modalizer) = &tabster.modalizer {
                modalizer.accept_element(&element, &mut state)
            } else {
                None
            }
        };

        if result.is_some() {
            state.skipped_focusable = Some(true);
        }

        if result.is_none() {
            result = if (state.accept_condition)(element.dyn_into().unwrap_throw()) {
                Some(*NodeFilterEnum::FilterAccept)
            } else {
                Some(*NodeFilterEnum::FilterSkip)
            };

            // if (
            //     result === NodeFilter.FILTER_SKIP &&
            //     this.isFocusable(element, false, true, true)
            // ) {
            //     state.skippedFocusable = true;
            // }
        }

        result.unwrap()
    }
}
