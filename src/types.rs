use std::sync::Arc;
use serde::{Deserialize, Serialize};
use web_sys::{
    js_sys::Function,
    wasm_bindgen::{prelude::Closure, JsCast, UnwrapThrowExt},
    Document, Element, HtmlElement, KeyboardEvent, MutationObserver, MutationRecord, Node,
    NodeFilter, TreeWalker, Window,
};

use crate::{
    groupper::Groupper, modalizer::Modalizer, mover::Mover, mutation_event::observe_mutations,
};

#[derive(Debug, Default)]
pub struct IgnoreKeydown {
    tab: Option<bool>,
    // Escape?: boolean;
    // Enter?: boolean;
    // ArrowUp?: boolean;
    // ArrowDown?: boolean;
    // ArrowLeft?: boolean;
    // ArrowRight?: boolean;
    // PageUp?: boolean;
    // PageDown?: boolean;
    // Home?: boolean;
    // End?: boolean;
}

impl IgnoreKeydown {
    pub fn get(&self, key: &str) -> Option<bool> {
        match key {
            "tab" => self.tab,
            _ => None,
        }
    }
}

pub struct FocusableProps {
    /// Do not determine an element's focusability based on aria-disabled.
    pub ignore_aria_disabled: Option<bool>,
    /// Exclude element (and all subelements) from Mover navigation.
    pub exclude_from_mover: Option<bool>,
    /// Prevents tabster from handling the keydown event
    pub ignore_keydown: Option<IgnoreKeydown>,
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
    pub groupper_before_mover: Option<bool>,
    /// Whether `dir='rtl'` is set on an ancestor
    pub rtl: Option<bool>,
    pub excluded_from_mover: Option<bool>,
    pub uncontrolled: Option<HtmlElement>,
    pub ignore_keydown: Box<dyn Fn(KeyboardEvent) -> bool>,
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

impl From<FindNextProps> for FindFocusableProps {
    fn from(value: FindNextProps) -> Self {
        Self {
            container: value.container,
            current_element: value.current_element,
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
    pub container: HtmlElement,
}

pub struct FindNextProps {
    /// The elemet to start from.
    pub current_element: Option<HtmlElement>,
    /// The container used for the search.
    pub container: HtmlElement,
}

pub struct FindAllProps {
    /// The container used for the search.
    container: HtmlElement,
}

pub struct RestoreFocusOrder {
    history: u32,
    deloser_default: u32,
    root_default: u32,
    deloser_first: u32,
    root_first: u32,
}

pub struct RootProps {
    restore_focus_order: Option<RestoreFocusOrder>,
}

pub struct TabsterProps {
    pub auto_root: Option<RootProps>,
    /// Custom getter for parent elements. Defaults to the default .parentElement call
    /// Currently only used to detect tabster contexts
    pub get_parent: Option<Box<dyn Fn(Node) -> Option<Node>>>,
}

pub struct TabsterCoreProps {
    pub auto_root: Option<RootProps>,
    /// Custom getter for parent elements. Defaults to the default .parentElement call
    /// Currently only used to detect tabster contexts
    pub get_parent: Option<Box<dyn Fn(Node) -> Option<Node>>>,
}

pub type ParentNode = Node;

pub trait DOMAPI {
    fn create_mutation_observer(
        callback: impl Fn(Vec<MutationRecord>, MutationObserver) + 'static,
    ) -> MutationObserver;

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

pub struct  TabsterAttributeOnElement {
    pub string: String,
    pub object: TabsterAttributeProps
}

#[derive(Default)]
pub struct TabsterOnElement {
    pub mover: Option<Mover>,
    pub groupper: Option<Groupper>,
    pub modalizer: Option<Modalizer>,
    pub focusable: Option<FocusableProps>,
    pub uncontrolled: Option<UncontrolledProps>,
}

pub struct TabsterElementStorageEntry {
    pub tabster: Option<Arc<TabsterOnElement>>,
    pub attr: Option<TabsterAttributeOnElement>,
                                                // aug?: TabsterAugmentedAttributes;
}

impl TabsterElementStorageEntry {
    pub fn new() -> Self {
        Self { tabster: None, attr: None }
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

pub struct InternalAPI {
    unobserve: Option<Box<dyn Fn()>>,
    doc: Document,
}

impl InternalAPI {
    pub fn new(win: Window) -> Self {
        Self {
            unobserve: None,
            doc: win.document().unwrap_throw(),
        }
    }
    pub fn stop_observer(&mut self) {
        if let Some(unobserve) = self.unobserve.take() {
            unobserve();
        }
    }

    pub fn resume_observer(&mut self, sync_state: bool) {
        self.unobserve = Some(observe_mutations(&self.doc));
    }
}

/// 0 | 1 | 2
pub type GroupperTabbability = u8;

#[derive(Serialize, Deserialize)]
pub struct GroupperProps {
    tabbability: Option<GroupperTabbability>,
    delegated: Option<bool>
    // This allows to tweak the groupper behaviour for the cases when
    // the groupper container is not focusable and groupper has Limited or LimitedTrapFocus
    // tabbability. By default, the groupper will automatically become active once the focus
    // goes to first focusable element inside the groupper during tabbing. When true, the
    // groupper will become active only after Enter is pressed on first focusable element
    // inside the groupper.
}

#[derive(Serialize, Deserialize)]
pub struct TabsterAttributeProps {
    pub groupper: Option<GroupperProps>,
}

impl TabsterAttributeProps {
    pub fn json_string(self) -> String {
        String::new()
    }
}

pub type TabsterDOMAttribute = (String, String);
