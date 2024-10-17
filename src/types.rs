use web_sys::{Document, HtmlElement, Node, NodeFilter, TreeWalker};

pub struct FindFocusableProps {
    /// The container used for the search.
    pub container: HtmlElement,
}

impl From<FindAllProps> for FindFocusableProps {
    fn from(value: FindAllProps) -> Self {
        Self {
            container: value.container,
        }
    }
}

pub struct FindAllProps {
    /// The container used for the search.
    container: HtmlElement,
}

pub trait DOMAPI {
    fn create_tree_walker(
        doc: Document,
        root: &Node,
        what_to_show: u32,
        filter: Option<&NodeFilter>,
    ) -> TreeWalker;
}

pub struct FocusableAcceptElementState {
    pub container: HtmlElement,
    pub found: Option<bool>,
}
