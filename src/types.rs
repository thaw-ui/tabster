use std::sync::Arc;
use web_sys::{Document, Element, HtmlElement, Node, NodeFilter, TreeWalker, Window};

pub struct FocusableProps {
    /// Do not determine an element's focusability based on aria-disabled.
    pub ignore_aria_disabled: Option<bool>,
}

pub struct UncontrolledProps {
    // Normally, even uncontrolled areas should not be completely uncontrolled
    // to be able to interact with the rest of the application properly.
    // For example, if an uncontrolled area implements something like
    // roving tabindex, it should be uncontrolled inside the area, but it
    // still should be able to be an organic part of the application.
    // However, in some cases, third party component might want to be able
    // to gain full control of the area, for example, if it implements
    // some custom trap focus logic.
    // `completely` indicates that uncontrolled area must gain full control over
    // Tab handling. If not set, Tabster might still handle Tab in the
    // uncontrolled area, when, for example, there is an inactive Modalizer
    // (that needs to be skipped) after the last focusable element of the
    // uncontrolled area.
    // WARNING: Use with caution, as it might break the normal keyboard navigation
    // between the uncontrolled area and the rest of the application.
    completely: Option<bool>,
}

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

pub struct TabsterContext {
    pub root: Root,
    pub uncontrolled: Option<HtmlElement>,
}

pub struct Root {}

pub struct ModalizerAPI {
    pub is_augmented: Box<dyn Fn(HtmlElement) -> bool>,
}

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
    /// Includes elements that can be focused programmatically.
    pub include_programmatically_focusable: Option<bool>,
    /// Ignore accessibility check.
    pub ignore_accessibility: Option<bool>,
    /// If true, find previous element instead of the next one.
    pub is_backward: Option<bool>,
    /// el: element visited.
    /// returns: if an element should be accepted.
    pub accept_condition: Option<Box<dyn Fn(HtmlElement) -> bool>>,
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
            include_programmatically_focusable: None,
            ignore_accessibility: None,
            is_backward: None,
            accept_condition: None,
            on_element: None,
        }
    }
}

impl From<FindAllProps> for FindFocusableProps {
    fn from(value: FindAllProps) -> Self {
        Self {
            container: value.container,
            current_element: None,
            include_programmatically_focusable: None,
            ignore_accessibility: None,
            is_backward: None,
            accept_condition: None,
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

    fn get_parent_element(element: Option<HtmlElement>) -> Option<HtmlElement>;

    fn node_contains(parent: Option<Node>, child: Option<Node>) -> bool;

    fn get_last_element_child(element: Option<Element>) -> Option<Element>;
}

pub type GetWindow = Box<dyn Fn() -> Window>;

pub struct FocusableAcceptElementState {
    pub container: HtmlElement,
    pub from: HtmlElement,
    pub from_ctx: Option<TabsterContext>,
    pub found: Option<bool>,
    pub found_element: Option<HtmlElement>,
    pub found_backward: Option<HtmlElement>,
    pub reject_elements_from: Option<HtmlElement>,
    pub accept_condition: Box<dyn Fn(HtmlElement) -> bool>,
    /// A flag that indicates that some focusable elements were skipped
    /// during the search and the found element is not the one the browser
    /// would normally focus if the user pressed Tab.
    pub skipped_focusable: Option<bool>,
}

pub struct TabsterOnElement {
    pub focusable: Option<FocusableProps>,
    pub uncontrolled: Option<UncontrolledProps>,
}

pub struct TabsterElementStorageEntry {
    pub tabster: Option<Arc<TabsterOnElement>>, // attr?: TabsterAttributeOnElement;
                                                // aug?: TabsterAugmentedAttributes;
}

impl TabsterElementStorageEntry {
    pub fn new() -> Self {
        Self { tabster: None }
    }

    pub fn is_empty(&self) -> bool {
        if self.tabster.is_some() {
            true
        } else {
            false
        }
    }
}

pub type TabsterElementStorage = TabsterElementStorageEntry;
