use web_sys::{Element, HtmlElement};

use crate::{
    dom_api::DOM,
    types::{self, DOMAPI},
    utils::{NodeFilterEnum, TabsterPart},
};
use std::{cell::RefMut, ops::Deref};

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

        if modalizer_user_id == current_modalizer.clone().map(|cm| cm.user_id.clone()) {
            None
        } else if modalizer_user_id.is_none()
            && current_modalizer
                .map(|cm| cm.get_props().is_always_accessible.unwrap_or_default())
                .unwrap_or_default()
        {
            None
        } else {
            state.skipped_focusable = Some(true);
            Some(*NodeFilterEnum::FilterSkip)
        }
    }
}
