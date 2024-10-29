use crate::{
    consts::TABSTER_DUMMY_INPUT_ATTRIBUTE_NAME,
    dom_api::DOM,
    tabster::TabsterCore,
    types::{self, GetWindow, DOMAPI},
    web::set_timeout,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    ops::Deref,
    sync::{Arc, OnceLock, RwLock},
};
use web_sys::{
    wasm_bindgen::{prelude::Closure, JsCast, UnwrapThrowExt},
    Document, Element, HtmlElement, Node, NodeFilter, TreeWalker,
};

pub struct WeakHTMLElement<T, D> {
    weak_ref: T,
    data: Option<D>,
}

impl<T, D: Clone> WeakHTMLElement<T, D> {
    fn new(get_window: GetWindow, element: T, data: Option<D>) -> Self {
        let context = get_instance_context(get_window);

        // let ref: TabsterWeakRef<T>;
        // if (context.WeakRef) {
        //     ref = new context.WeakRef(element);
        // } else {
        //     ref = new FakeWeakRef(element);
        //     context.fakeWeakRefs.push(ref);
        // }

        Self {
            weak_ref: element,
            data,
        }
    }

    // fn get(&self) -> Option<T> {
    //     const ref = this._ref;
    //     let element: T | undefined;

    //     if (ref) {
    //         element = ref.deref();

    //         if (!element) {
    //             delete this._ref;
    //         }
    //     }

    //     return element;
    // }

    fn get_data(&self) -> Option<D> {
        self.data.clone()
    }
}

static LAST_TABSTER_PART_ID: OnceLock<RwLock<usize>> = OnceLock::new();

pub struct TabsterPart<P> {
    pub id: String,
    pub tabster: Arc<RefCell<TabsterCore>>,
    element: HtmlElement,
    pub props: P,
}

impl<P> TabsterPart<P> {
    pub fn new(tabster: Arc<RefCell<TabsterCore>>, element: HtmlElement, props: P) -> Self {
        let last_tabster_part_id = LAST_TABSTER_PART_ID.get_or_init(Default::default);
        let id = *last_tabster_part_id.read().unwrap_throw() + 1;
        *last_tabster_part_id.write().unwrap_throw() = id;

        Self {
            id: format!("i{}", id),
            tabster,
            element,
            props,
        }
    }

    pub fn get_element(&self) -> Option<HtmlElement> {
        Some(self.element.clone())
    }
}

pub struct DummyInputManager {
    instance: Option<Arc<RefCell<DummyInputManagerCore>>>,
    element: HtmlElement,
}

impl DummyInputManager {
    pub fn new(
        tabster: Arc<RefCell<TabsterCore>>,
        element: HtmlElement,
        sys: Option<types::SysProps>,
    ) -> Self {
        let instance = DummyInputManagerCore::new(tabster, element.clone(), sys);
        Self {
            instance: Some(instance),
            element,
        }
    }
}

struct DummyInputManagerCore {
    add_timer: Arc<RefCell<Option<i32>>>,
    get_window: Arc<GetWindow>,
    element: Option<HtmlElement>,
    is_outside: bool,
    first_dummy: Option<DummyInput>,
    last_dummy: Option<DummyInput>,
}

impl DummyInputManagerCore {
    fn new(
        tabster: Arc<RefCell<TabsterCore>>,
        element: HtmlElement,
        sys: Option<types::SysProps>,
    ) -> Arc<RefCell<Self>> {
        let tabster = tabster.borrow();
        let get_window = &tabster.get_window;
        let first_dummy = DummyInput::new(get_window.clone());
        let last_dummy = DummyInput::new(get_window.clone());
        let tag_name = element.tag_name();

        let this = Arc::new(RefCell::new(Self {
            add_timer: Default::default(),
            get_window: get_window.clone(),
            element: Some(element),
            is_outside: false,
            first_dummy: Some(first_dummy),
            last_dummy: Some(last_dummy),
        }));

        Self::add_dummy_inputs(this.clone());

        this
    }

    /// Adds dummy inputs as the first and last child of the given element
    /// Called each time the children under the element is mutated
    fn add_dummy_inputs(this: Arc<RefCell<DummyInputManagerCore>>) {
        let this_ = this.clone();
        let this = this.borrow();
        let add_timer = this.add_timer.clone();
        let mut add_timer_ref = this.add_timer.borrow_mut();
        if add_timer_ref.is_some() {
            return;
        }

        let timer = set_timeout(
            &(this.get_window)(),
            move || {
                let mut add_timer = add_timer.borrow_mut();
                *add_timer = None;

                let this = this_.borrow();
                this.ensure_position();

                // if (__DEV__) {
                //     this._firstDummy &&
                //         setDummyInputDebugValue(this._firstDummy, this._wrappers);
                //     this._lastDummy &&
                //         setDummyInputDebugValue(this._lastDummy, this._wrappers);
                // }

                // this._addTransformOffsets();
            },
            0,
        );
        *add_timer_ref = Some(timer);
    }

    fn ensure_position(&self) {
        let element = self.element.clone();
        let first_dummy_input = if let Some(first_dummy) = &self.first_dummy {
            first_dummy.input.clone()
        } else {
            None
        };
        let last_dummy_input = if let Some(last_dummy) = &self.last_dummy {
            last_dummy.input.clone()
        } else {
            None
        };

        let Some(element) = element else {
            return;
        };
        let Some(first_dummy_input) = first_dummy_input else {
            return;
        };
        let Some(last_dummy_input) = last_dummy_input else {
            return;
        };
        // if (this._isOutside) {
        // }

        if DOM::get_last_element_child(Some(element.clone().dyn_into().unwrap_throw()))
            != Some(last_dummy_input.clone().dyn_into().unwrap_throw())
        {
            DOM::append_child(element.clone().into(), last_dummy_input.clone().into());
        }

        if let Some(first_element_child) = DOM::get_first_element_child(Some(element.into())) {
            if first_element_child != *first_dummy_input {
                if let Some(parent_node) = first_element_child.parent_node() {
                    DOM::insert_before(
                        parent_node,
                        first_dummy_input.into(),
                        Some(first_element_child.into()),
                    );
                }
            }
        }
    }
}

struct DummyInput {
    input: Option<HtmlElement>,
}

impl DummyInput {
    fn new(get_window: Arc<GetWindow>) -> Self {
        let win = get_window();
        let input: HtmlElement = win
            .document()
            .unwrap_throw()
            .create_element("i")
            .unwrap_throw()
            .dyn_into()
            .unwrap_throw();

        input.set_tab_index(0);
        input.set_attribute("role", "none").unwrap_throw();

        input
            .set_attribute(TABSTER_DUMMY_INPUT_ATTRIBUTE_NAME, "")
            .unwrap_throw();
        input.set_attribute("aria-hidden", "true").unwrap_throw();
        input.set_attribute("style", "position:fixed;width:1px;height:1px;opacity:0.001;z-index:-1;content-visibility:hidden").unwrap_throw();

        // makeFocusIgnored(input);

        // this.input = input;
        // this.isFirst = props.isFirst;
        // this.isOutside = isOutside;
        // this._isPhantom = props.isPhantom ?? false;
        // this._fixedTarget = fixedTarget;

        // input.addEventListener("focusin", this._focusIn);
        // input.addEventListener("focusout", this._focusOut);

        // (input as HTMLElementWithDummyContainer).__tabsterDummyContainer =
        //     element;

        // if (this._isPhantom) {
        //     this._disposeTimer = win.setTimeout(() => {
        //         delete this._disposeTimer;
        //         this.dispose();
        //     }, 0);

        //     this._clearDisposeTimeout = () => {
        //         if (this._disposeTimer) {
        //             win.clearTimeout(this._disposeTimer);
        //             delete this._disposeTimer;
        //         }

        //         delete this._clearDisposeTimeout;
        //     };
        // }

        Self { input: Some(input) }
    }
}

pub fn create_element_tree_walker(
    doc: &Document,
    root: &Node,
    accept_node: impl Fn(Node) -> u32 + 'static,
) -> Option<TreeWalker> {
    if root.node_type() != Node::ELEMENT_NODE {
        return None;
    }
    let node_filter = NodeFilter::new();
    let cb: Closure<dyn Fn(Node) -> u32> = Closure::new(accept_node);
    node_filter.set_accept_node(cb.as_ref().unchecked_ref());

    Some(DOM::create_tree_walker(
        doc,
        root,
        *NodeFilterEnum::ShowElement,
        Some(&node_filter),
    ))
}

pub enum NodeFilterEnum {
    FilterAccept,
    FilterReject,
    FilterSkip,
    ShowElement,
}

impl Deref for NodeFilterEnum {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::FilterAccept => &1,
            Self::FilterReject => &2,
            Self::FilterSkip => &3,
            Self::ShowElement => &0x1,
        }
    }
}

pub fn get_last_child(container: HtmlElement) -> Option<HtmlElement> {
    let mut last_child: Option<HtmlElement> = None;
    let mut el = DOM::get_last_element_child(Some(container.dyn_into().unwrap_throw()));
    loop {
        if el.is_none() {
            break;
        }
        last_child = el.clone().map(|el| el.dyn_into().unwrap_throw());
        el = DOM::get_last_element_child(el);
    }
    last_child
}

static TABSTER_INSTANCE_CONTEXT: OnceLock<RwLock<HashMap<String, Arc<InstanceContext>>>> =
    OnceLock::new();

// struct InternalBasics {

// }

struct InstanceContext {
    // elementByUId: { [uid: string]: WeakHTMLElement<HTMLElementWithUID> };
    // basics: InternalBasics,
    // WeakRef?: WeakRefConstructor;
    // containerBoundingRectCache: {
    //     [id: string]: {
    //         rect: TabsterDOMRect;
    //         element: HTMLElementWithBoundingRectCacheId;
    //     };
    // };
    last_container_bounding_rect_cache_id: i32,
    container_bounding_rect_cache_timer: Option<i32>,
    // fakeWeakRefs: TabsterWeakRef<unknown>[];
    fake_weak_refs_timer: Option<i32>,
    fake_weak_refs_started: bool,
}

pub fn get_instance_context(get_window: GetWindow) -> Arc<InstanceContext> {
    // interface WindowWithUtilsConext extends Window {
    //     __tabsterInstanceContext?: InstanceContext;
    //     Promise: PromiseConstructor;
    //     WeakRef: WeakRefConstructor;
    // }
    let win = get_window();
    let tabster_instance_context = TABSTER_INSTANCE_CONTEXT.get_or_init(Default::default);
    if let Some(obj) = win.get("__tabsterInstanceContext") {
        if let Some(key) = obj.as_string() {
            if let Some(ctx) = tabster_instance_context
                .read()
                .unwrap_throw()
                .get(&key)
                .cloned()
            {
                return ctx;
            }
        }
    }

    // ctx = {
    //     elementByUId: {},
    //     basics: {
    //         Promise: win.Promise || undefined,
    //         WeakRef: win.WeakRef || undefined,
    //     },
    //     containerBoundingRectCache: {},
    //     fakeWeakRefs: [],
    // };
    let ctx = Arc::new(InstanceContext {
        last_container_bounding_rect_cache_id: 0,
        container_bounding_rect_cache_timer: None,
        fake_weak_refs_timer: None,
        fake_weak_refs_started: false,
    });
    let key = uuid::Uuid::new_v4().to_string();
    tabster_instance_context
        .write()
        .unwrap_throw()
        .insert(String::new(), ctx.clone());
    web_sys::js_sys::Reflect::set(
        &win,
        &web_sys::wasm_bindgen::JsValue::from_str("__tabsterInstanceContext"),
        &web_sys::wasm_bindgen::JsValue::from_str(&key),
    )
    .unwrap_throw();
    ctx
}

// pub fn  create_weak_map<K extends object, V>(win: Window) -> WeakMap {
//     const ctx = (win as WindowWithUtilsConext).__tabsterInstanceContext;
//     return new (ctx?.basics.WeakMap || WeakMap)();
// }

pub fn matches_selector(element: HtmlElement, selector: String) -> bool {
    element.matches(&selector).unwrap_throw()
}

pub fn is_display_none(element: HtmlElement) -> bool {
    let element_document = element.owner_document().unwrap_throw();

    let computed_style = {
        let Some(default_view) = element_document.default_view() else {
            return false;
        };
        default_view.get_computed_style(&element).unwrap_throw()
    };

    // offsetParent is null for elements with display:none, display:fixed and for <body>.
    if element.offset_parent().is_none()
        && element_document.body().as_ref() != Some(&element)
        && computed_style
            .as_ref()
            .map(|c| c.get_property_value("position").unwrap_throw())
            != Some("fixed".into())
    {
        return true;
    }

    // For our purposes of looking for focusable elements, visibility:hidden has the same
    // effect as display:none.
    if computed_style
        .as_ref()
        .map(|c| c.get_property_value("visibility").unwrap_throw())
        == Some("hidden".into())
    {
        return true;
    }

    // if an element has display: fixed, we need to check if it is also hidden with CSS,
    // or within a parent hidden with CSS
    if computed_style
        .as_ref()
        .map(|c| c.get_property_value("position").unwrap_throw())
        == Some("fixed".into())
    {
        if computed_style
            .as_ref()
            .map(|c| c.get_property_value("display").unwrap_throw())
            == Some("none".into())
        {
            return true;
        }

        let Some(parent_element) = element.parent_element() else {
            return false;
        };

        let Ok(parent_element) = parent_element.dyn_into::<HtmlElement>() else {
            return false;
        };

        if parent_element.offset_parent().is_none()
            && element_document.body().map(|b| b.into()) != element.parent_element()
        {
            return true;
        }
    }

    false
}

/// If the passed element is Tabster dummy input, returns the container element this dummy input belongs to.
/// element: Element to check for being dummy input.
/// returns: Dummy input container element (if the passed element is a dummy input) or null.
pub fn get_dummy_input_container(element: Option<HtmlElement>) -> Option<HtmlElement> {
    // return (
    //     (
    //         element as HTMLElementWithDummyContainer | null | undefined
    //     )?.__tabsterDummyContainer?.get() || null
    // );
    None
}
