use web_sys::{wasm_bindgen::UnwrapThrowExt, Element, HtmlElement};

use crate::{
    dom_api::DOM,
    root::RootAPI,
    types::{self, DOMAPI},
    utils::{NodeFilterEnum, TabsterPart},
};
use std::{
    cell::{RefCell, RefMut},
    ops::Deref,
    sync::Arc,
};

pub type ArcCellModalizer = Arc<RefCell<Modalizer>>;

pub struct Modalizer {
    part: TabsterPart<types::ModalizerProps>,
    pub user_id: String,
}

impl Deref for Modalizer {
    type Target = TabsterPart<types::ModalizerProps>;

    fn deref(&self) -> &Self::Target {
        &self.part
    }
}

impl Modalizer {
    pub(crate) fn find_next_tabbable(
        &mut self,
        current_element: Option<HtmlElement>,
        reference_element: Option<HtmlElement>,
        is_backward: Option<bool>,
        ignore_accessibility: Option<bool>,
    ) -> Option<types::NextTabbable> {
        if self.get_element().is_none() {
            return None;
        }

        let container = current_element
            .clone()
            .map(|current_element| {
                RootAPI::get_root(&self.tabster, current_element)
                    .map(|root| root.get_element())
                    .flatten()
            })
            .flatten();

        let mut next = None::<HtmlElement>;
        let mut out_of_dom_order = false;
        let mut uncontrolled = None::<HtmlElement>;

        if let Some(container) = container {
            let find_props = types::FindNextProps {
                container: container.clone(),
                current_element: current_element.clone(),
                reference_element,
                ignore_accessibility,
                use_active_modalizer: Some(true),
            };

            let mut find_props_out = types::FindFocusableOutputProps::default();

            let tabster = self.tabster.borrow();
            let focusable = tabster.focusable.clone().unwrap_throw();
            let mut focusable = focusable.borrow_mut();
            next = if is_backward.unwrap_or_default() {
                focusable.find_prev(find_props, &mut find_props_out)
            } else {
                focusable.find_next(find_props, &mut find_props_out)
            };

            let active_id = if let Some(modalizer) = &tabster.modalizer {
                modalizer.active_id.clone()
            } else {
                None
            };

            if next.is_none() && self.props.is_trapped.unwrap_or_default() && active_id.is_some() {
                let find_props = types::FindFirstProps {
                    container,
                    ignore_accessibility,
                    use_active_modalizer: Some(true),
                };

                next = if is_backward.unwrap_or_default() {
                    focusable.find_last(find_props, &mut find_props_out)
                } else {
                    focusable.find_first(find_props, &mut find_props_out)
                };

                if next.is_none() {
                    next = current_element;
                }

                out_of_dom_order = true;
            } else {
                out_of_dom_order = find_props_out.out_of_dom_order.unwrap_or_default();
            }

            uncontrolled = find_props_out.uncontrolled;
        }

        Some(types::NextTabbable {
            element: next,
            uncontrolled,
            out_of_dom_order: Some(out_of_dom_order),
        })
    }
}

pub struct ModalizerAPI {
    pub active_id: Option<String>,
    pub is_augmented: Box<dyn Fn(Element) -> bool>,
    pub active_elements: Vec<HtmlElement>,
}

impl ModalizerAPI {
    pub fn accept_element(
        &self,
        element: &Element,
        state: &mut RefMut<'_, types::FocusableAcceptElementState>,
    ) -> Option<u32> {
        let modalizer_user_id = state.modalizer_user_id.clone();
        let current_modalizer = state.current_ctx.clone().map(|c| c.modalizer).flatten();

        if modalizer_user_id.is_some() {
            for el in self.active_elements.iter() {
                if DOM::node_contains(Some(element.clone().into()), Some(el.clone().into()))
                    || **el == *element
                {
                    // We have a part of currently active modalizer somewhere deeper in the DOM,
                    // skipping all other checks.
                    return Some(*NodeFilterEnum::FilterSkip);
                }
            }
        }

        if modalizer_user_id
            == current_modalizer
                .clone()
                .map(|cm| cm.borrow().user_id.clone())
        {
            None
        } else if modalizer_user_id.is_none()
            && current_modalizer
                .map(|cm| {
                    cm.borrow()
                        .get_props()
                        .is_always_accessible
                        .unwrap_or_default()
                })
                .unwrap_or_default()
        {
            None
        } else {
            state.skipped_focusable = Some(true);
            Some(*NodeFilterEnum::FilterSkip)
        }
    }
}
