use std::collections::HashMap;

use web_sys::{Document, Element, HtmlElement, Node, NodeFilter, TreeWalker, Window};

#[derive(Debug, Default)]
pub struct GetTabsterContextOptions {
    /// Should visit **all** element ancestors to verify if `dir='rtl'` is set
    pub check_rtl: Option<bool>,
    /// The element to start computing the context from. Useful when dealing
    /// with nested structures. For example, if we have an element inside a groupper
    /// inside another groupper, the `groupper` prop in this element's contexts will
    /// be the inner groupper, but when we pass the inner groupper's parent element
    /// as `referenceElement`, the context groupper will be the outer one. Having
    /// this option simplifies searching for the next tabbable element in the
    /// environment of nested movers and grouppers.
    pub reference_element: Option<HtmlElement>,
}

#[derive(Debug, Clone)]
pub struct TabsterContext {
    pub uncontrolled: Option<HtmlElement>,
}

pub struct Root {}

#[derive(Debug, Default)]
pub struct FindFocusableOutputProps {
    /// An output parameter. Will be true after the find_next/find_prev() call if some focusable
    /// elements were skipped during the search and the result element not immediately next
    /// focusable after the currentElement.
    pub out_of_dom_order: Option<bool>,
    /// An output parameter. Will be true if the found element is uncontrolled.
    pub uncontrolled: Option<HtmlElement>,
}

pub struct FindFocusableProps {
    /// The container used for the search.
    pub container: HtmlElement,
    /// The elemet to start from.
    pub current_element: Option<HtmlElement>,
    /// If true, find previous element instead of the next one.
    pub is_backward: Option<bool>,
    /// A callback that will be called for every focusable element found during findAll().
    /// If false is returned from this callback, the search will stop.
    pub on_element: Option<FindElementCallback>,
}

type FindElementCallback = Box<dyn Fn(HtmlElement) -> bool>;

impl From<FindFirstProps> for FindFocusableProps {
    fn from(value: FindFirstProps) -> Self {
        Self {
            container: value.container,
            current_element: None,
            is_backward: None,
            on_element: None,
        }
    }
}

impl From<FindAllProps> for FindFocusableProps {
    fn from(value: FindAllProps) -> Self {
        Self {
            container: value.container,
            current_element: None,
            is_backward: None,
            on_element: None,
        }
    }
}

pub struct FindFirstProps {
    /// The container used for the search.
    container: HtmlElement,
}

pub struct FindAllProps {
    /// The container used for the search.
    container: HtmlElement,
}

pub struct TabsterCoreProps {
    /// Custom getter for parent elements. Defaults to the default .parentElement call
    /// Currently only used to detect tabster contexts
    pub get_parent: Option<Box<dyn Fn(Node) -> Option<Node>>>,
}

pub type ParentNode = Node;

pub trait DOMAPI {
    fn create_tree_walker(
        doc: Document,
        root: &Node,
        what_to_show: u32,
        filter: Option<&NodeFilter>,
    ) -> TreeWalker;

    fn get_parent_node(node: Option<Node>) -> Option<ParentNode>;

    fn get_last_element_child(element: Option<Element>) -> Option<Element>;
}

pub type GetWindow = Box<dyn Fn() -> Window>;

#[derive(Debug, Clone)]
pub struct FocusableAcceptElementState {
    pub container: HtmlElement,
    pub from: HtmlElement,
    pub from_ctx: Option<TabsterContext>,
    pub found: Option<bool>,
    pub found_element: Option<HtmlElement>,
    pub found_backward: Option<HtmlElement>,
    /// A flag that indicates that some focusable elements were skipped
    /// during the search and the found element is not the one the browser
    /// would normally focus if the user pressed Tab.
    pub skipped_focusable: Option<bool>,
}

pub struct TabsterOnElement {}

pub struct TabsterElementStorageEntry {
    pub tabster: Option<TabsterOnElement>, // attr?: TabsterAttributeOnElement;
                                           // aug?: TabsterAugmentedAttributes;
}

pub type TabsterElementStorage = HashMap<String, TabsterElementStorageEntry>;
