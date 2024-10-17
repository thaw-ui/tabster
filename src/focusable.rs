use crate::{
    types::{FindAllProps, FindFocusableProps, FocusableAcceptElementState},
    utils::{create_element_tree_walker, NodeFilterEnum},
};
use web_sys::{
    wasm_bindgen::{JsCast, UnwrapThrowExt},
    HtmlElement,
};

pub struct FocusableAPI {}

impl FocusableAPI {
    pub fn new() -> Self {
        Self {}
    }

    pub fn find_all(&self, options: FindAllProps) {
        self.find_elements(true, options.into());
    }

    fn find_element() {}

    fn find_elements(
        &self,
        is_find_all: bool,
        options: FindFocusableProps,
    ) -> Option<Vec<HtmlElement>> {
        let FindFocusableProps { container } = options;

        let elements = Vec::<HtmlElement>::new();

        let accept_element_state = FocusableAcceptElementState {
            container: container.clone(),
            found: None,
        };
        let walker = create_element_tree_walker(
            container.owner_document().unwrap_throw(),
            &container,
            move |node| Self::accept_element(node.dyn_into().unwrap_throw(), &accept_element_state),
        );

        None
    }

    fn accept_element(element: HtmlElement, state: &FocusableAcceptElementState) -> u32 {
        if matches!(state.found, Some(true)) {
            return *NodeFilterEnum::FilterAccept;
        }
        todo!()
    }
}
