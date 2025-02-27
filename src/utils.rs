use crate::{
    console_log,
    consts::TABSTER_DUMMY_INPUT_ATTRIBUTE_NAME,
    dom_api::DOM,
    tabster::TabsterCore,
    types::{self, GetWindow, DOMAPI},
    web::set_timeout,
    SysDummyInputsPositions,
};
use send_wrapper::SendWrapper;
use std::{
    cell::RefCell,
    collections::HashMap,
    ops::Deref,
    sync::{Arc, LazyLock, OnceLock, RwLock},
};
use web_sys::{
    js_sys::{self, Reflect, Uint32Array},
    wasm_bindgen::{
        self,
        prelude::{wasm_bindgen, Closure},
        JsCast, JsValue, UnwrapThrowExt,
    },
    Document, Element, HtmlElement, HtmlInputElement, Node, NodeFilter, TreeWalker, Window,
};

#[derive(Clone)]
struct FakeWeakRef<T: DerefHtmlElement + Clone> {
    target: Option<T>,
}

impl<T: DerefHtmlElement + Clone> FakeWeakRef<T> {
    pub fn new(target: Option<T>) -> Self {
        Self { target }
    }
}

pub trait DerefHtmlElement {
    fn deref(&self) -> Option<HtmlElement>;
}

impl<T: DerefHtmlElement + Clone> DerefHtmlElement for FakeWeakRef<T> {
    fn deref(&self) -> Option<HtmlElement> {
        if let Some(target) = &self.target {
            target.deref()
        } else {
            None
        }
    }
}

impl DerefHtmlElement for HtmlElement {
    fn deref(&self) -> Option<HtmlElement> {
        Some(self.clone())
    }
}

pub struct WeakHTMLElement<T: DerefHtmlElement + Clone = HtmlElement, D = ()> {
    weak_ref: RefCell<Option<FakeWeakRef<T>>>,
    data: Option<D>,
}

impl<T: DerefHtmlElement + Clone, D: Clone> Clone for WeakHTMLElement<T, D> {
    fn clone(&self) -> Self {
        Self {
            weak_ref: self.weak_ref.clone(),
            data: self.data.clone(),
        }
    }
}

impl<T: DerefHtmlElement + Clone + 'static, D: Clone> WeakHTMLElement<T, D> {
    pub fn new(get_window: Arc<GetWindow>, element: T, data: Option<D>) -> Self {
        let context = get_instance_context(&get_window);

        let weak_ref = FakeWeakRef::new(Some(element));
        context
            .fake_weak_refs
            .write()
            .unwrap()
            .push(SendWrapper::new(Box::new(weak_ref.clone())));

        Self {
            weak_ref: Some(weak_ref).into(),
            data,
        }
    }

    pub fn get(&self) -> Option<T> {
        let mut element = None::<T>;

        if let Some(weak_ref) = self.weak_ref.borrow().as_ref() {
            element = weak_ref.target.clone();

            if element.is_none() {
                *self.weak_ref.borrow_mut() = None;
            }
        }

        element
    }

    fn get_data(&self) -> Option<D> {
        self.data.clone()
    }
}

pub fn should_ignore_focus(element: &Element) -> bool {
    false
    // return !!(element as FocusedElementWithIgnoreFlag).__shouldIgnoreFocus;
}

fn to_base36(num: u32) -> String {
    const CHARS: &[char] = &[
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H',
        'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    ];

    if num == 0 {
        return "0".to_string();
    }

    let mut result = String::new();
    let mut n = num;

    while n > 0 {
        let rem = n % 36;
        result.insert(0, CHARS[rem as usize]);
        n /= 36;
    }

    result
}

static UID_COUNTER: LazyLock<RwLock<u32>> = LazyLock::new(|| Default::default());

fn get_uid(wnd: Window) -> String {
    let rnd = Uint32Array::new(&JsValue::from(4));

    if let Ok(crypto) = wnd.crypto() {
        crypto
            .get_random_values_with_array_buffer_view(&rnd)
            .unwrap_throw();
    } else {
        for i in 0..rnd.length() {
            // 4294967295 == 0xffffffff
            rnd.set_index(i, (4294967295.0 * js_sys::Math::random()) as u32);
        }
    }

    let mut srnd: Vec<String> = vec![];

    for i in 0..rnd.length() {
        srnd.push(to_base36(rnd.get_index(i)));
    }

    srnd.push("|".to_string());
    let mut uid_counter = UID_COUNTER.write().unwrap_throw();
    *uid_counter += 1;
    srnd.push(to_base36(*uid_counter));
    srnd.push("|".to_string());
    let date = js_sys::Date::now();
    srnd.push(to_base36(date as u32));

    srnd.join("")
}

pub fn get_element_uid(get_window: &Arc<GetWindow>, element: &HtmlElement) -> String {
    let context = get_instance_context(get_window);
    let uid = Reflect::get(element, &JsValue::from_str("__tabsterElementUID"))
        .unwrap_throw()
        .as_string();

    let uid = if let Some(uid) = uid {
        uid
    } else {
        let uid = get_uid(get_window());
        Reflect::set(
            element,
            &JsValue::from_str("__tabsterElementUID"),
            &JsValue::from(uid.clone()),
        )
        .unwrap_throw();
        uid
    };

    if !context
        .element_by_uid
        .read()
        .unwrap_throw()
        .contains_key(&uid)
        && element
            .owner_document()
            .map(|doc| doc.body())
            .flatten()
            .map_or(false, move |body| body.contains(Some(&element)))
    {
        // context.elementByUId[uid] = new WeakHTMLElement(getWindow, element);
    }

    uid
}

static LAST_TABSTER_PART_ID: OnceLock<RwLock<usize>> = OnceLock::new();

pub struct TabsterPart<P> {
    pub id: String,
    pub tabster: Arc<RefCell<TabsterCore>>,
    pub(crate) _element: WeakHTMLElement,
    pub props: P,
}

impl<P> TabsterPart<P> {
    pub fn new(tabster: Arc<RefCell<TabsterCore>>, element: HtmlElement, props: P) -> Self {
        let last_tabster_part_id = LAST_TABSTER_PART_ID.get_or_init(Default::default);
        let id = *last_tabster_part_id.read().unwrap_throw() + 1;
        *last_tabster_part_id.write().unwrap_throw() = id;
        let element = WeakHTMLElement::new(tabster.borrow().get_window.clone(), element, None);

        Self {
            id: format!("i{}", id),
            tabster,
            _element: element,
            props,
        }
    }

    pub fn get_element(&self) -> Option<HtmlElement> {
        self._element.get()
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn get_props(&self) -> &P {
        &self.props
    }

    pub fn set_props(&mut self, props: P) {
        self.props = props;
    }
}

pub type DummyInputFocusCallback = Box<dyn Fn(DummyInput, bool, Option<HtmlElement>)>;

pub struct DummyInputManager {
    instance: Option<Arc<RefCell<DummyInputManagerCore>>>,
    on_focus_in: Option<DummyInputFocusCallback>,
    on_focus_out: Option<DummyInputFocusCallback>,
    element: WeakHTMLElement,
}

impl DummyInputManager {
    pub fn new(
        tabster: Arc<RefCell<TabsterCore>>,
        element: WeakHTMLElement,
        sys: Option<types::SysProps>,
        outside_by_default: Option<bool>,
    ) -> Self {
        console_log!("DummyInputManager::new");
        let instance =
            DummyInputManagerCore::new(tabster, element.clone(), sys, outside_by_default);
        Self {
            instance: Some(instance),
            on_focus_in: None,
            on_focus_out: None,
            element,
        }
    }

    pub fn set_handlers(
        &mut self,
        on_focus_in: Option<DummyInputFocusCallback>,
        on_focus_out: Option<DummyInputFocusCallback>,
    ) {
        self.on_focus_in = on_focus_in;
        self.on_focus_out = on_focus_out;
    }
}

struct DummyInputManagerCore {
    add_timer: Arc<RefCell<Option<i32>>>,
    get_window: Arc<GetWindow>,
    element: Option<WeakHTMLElement>,
    is_outside: bool,
    first_dummy: Option<DummyInput>,
    last_dummy: Option<DummyInput>,
}

impl DummyInputManagerCore {
    fn new(
        tabster: Arc<RefCell<TabsterCore>>,
        element: WeakHTMLElement,
        sys: Option<types::SysProps>,
        outside_by_default: Option<bool>,
    ) -> Arc<RefCell<Self>> {
        let el = element.get().unwrap_throw();

        let forced_dummy_position = if let Some(sys) = sys.as_ref() {
            sys.dummy_inputs_position
        } else {
            None
        };
        let tag_name = el.tag_name();
        let is_outside = if let Some(forced_dummy_position) = forced_dummy_position {
            forced_dummy_position == *SysDummyInputsPositions::Outside
        } else {
            (outside_by_default.unwrap_or_default()
                || tag_name == "UL"
                || tag_name == "OL"
                || tag_name == "TABLE")
                && !(tag_name == "LI" || tag_name == "TD" || tag_name == "TH")
        };

        let tabster = tabster.borrow();
        let get_window = &tabster.get_window;
        let first_dummy = DummyInput::new(
            get_window.clone(),
            is_outside,
            DummyInputProps {
                is_phantom: None,
                is_first: true,
            },
            Some(element.clone()),
        );
        let last_dummy = DummyInput::new(
            get_window.clone(),
            is_outside,
            DummyInputProps {
                is_phantom: None,
                is_first: true,
            },
            Some(element.clone()),
        );

        let this = Arc::new(RefCell::new(Self {
            add_timer: Default::default(),
            get_window: get_window.clone(),
            element: Some(element),
            is_outside,
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
        let element = self.element.clone().map(|e| e.get()).flatten();
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
        if self.is_outside {
            let element_parent = DOM::get_parent_node(Some(element.clone().into()));
            if let Some(element_parent) = element_parent {
                let next_sibling = DOM::get_next_sibling(Some(element.clone().into()));

                if next_sibling != Some(last_dummy_input.clone().into()) {
                    DOM::insert_before(
                        element_parent.clone(),
                        last_dummy_input.into(),
                        next_sibling,
                    );
                }

                if DOM::get_previous_element_sibling(Some(element.clone().into()))
                    != Some(first_dummy_input.clone().into())
                {
                    DOM::insert_before(
                        element_parent,
                        first_dummy_input.into(),
                        Some(element.into()),
                    );
                }
            }
        } else {
            if DOM::get_last_element_child(Some(element.clone().dyn_into().unwrap_throw()))
                != Some(last_dummy_input.clone().dyn_into().unwrap_throw())
            {
                DOM::append_child(element.clone().into(), last_dummy_input.clone().into());
            }

            if let Some(first_element_child) = DOM::get_first_child(Some(element.into())) {
                if first_element_child != **first_dummy_input {
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
}

pub struct DummyInputProps {
    /// The input is created to be used only once and autoremoved when focused.
    is_phantom: Option<bool>,
    /// Whether the input is before or after the content it is guarding.
    is_first: bool,
}

pub struct DummyInput {
    is_phantom: bool,
    pub input: Option<HtmlElement>,
    is_first: bool,
    pub is_outside: bool,
}

impl DummyInput {
    fn new(
        get_window: Arc<GetWindow>,
        is_outside: bool,
        props: DummyInputProps,
        element: Option<WeakHTMLElement>,
    ) -> Self {
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

        console_log!("DummyInput::new");
        // makeFocusIgnored(input);

        let is_phantom = props.is_phantom.unwrap_or_default();
        // this._fixedTarget = fixedTarget;

        // input.addEventListener("focusin", this._focusIn);
        // input.addEventListener("focusout", this._focusOut);

        // (input as HTMLElementWithDummyContainer).__tabsterDummyContainer =
        //     element;

        if is_phantom {
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
        }

        Self {
            input: Some(input),
            is_first: props.is_first,
            is_phantom,
            is_outside,
        }
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
    let cb = cb.into_js_value();
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

pub fn get_adjacent_element(from: HtmlElement, prev: Option<bool>) -> Option<HtmlElement> {
    let mut cur = Some(from);
    let mut adjacent = None::<HtmlElement>;

    loop {
        let Some(new_cur) = cur else {
            break;
        };
        if adjacent.is_some() {
            break;
        }
        adjacent = if prev.unwrap_or_default() {
            DOM::get_previous_element_sibling(Some(new_cur.clone().into()))
                .map(|e| e.dyn_into().unwrap_throw())
        } else {
            DOM::get_next_element_sibling(Some(new_cur.clone().into()))
                .map(|e| e.dyn_into().unwrap_throw())
        };

        cur = DOM::get_parent_element(Some(new_cur));
    }

    adjacent
}

pub fn get_last_child(container: &HtmlElement) -> Option<HtmlElement> {
    let mut last_child: Option<HtmlElement> = None;
    let mut el = DOM::get_last_element_child(Some(container.clone().dyn_into().unwrap_throw()));
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

pub struct InstanceContext {
    pub element_by_uid: Arc<RwLock<HashMap<String, String>>>,
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
    fake_weak_refs: RwLock<Vec<SendWrapper<Box<dyn DerefHtmlElement>>>>,
    fake_weak_refs_timer: Option<i32>,
    fake_weak_refs_started: bool,
}

pub fn get_instance_context(get_window: &Arc<GetWindow>) -> Arc<InstanceContext> {
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
    // };
    let ctx = Arc::new(InstanceContext {
        element_by_uid: Default::default(),
        last_container_bounding_rect_cache_id: 0,
        container_bounding_rect_cache_timer: None,
        fake_weak_refs: Default::default(),
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

pub fn matches_selector(element: &Element, selector: &str) -> bool {
    element.matches(selector).unwrap_throw()
}

pub fn is_display_none(element: Element) -> bool {
    let element_document = element.owner_document().unwrap_throw();

    let computed_style = if let Some(default_view) = element_document.default_view() {
        default_view.get_computed_style(&element).unwrap_throw()
    } else {
        None
    };

    // offsetParent is null for elements with display:none, display:fixed and for <body>.
    if element.clone().dyn_into::<HtmlElement>().is_err()
        || element
            .clone()
            .dyn_into::<HtmlElement>()
            .unwrap_throw()
            .offset_parent()
            .is_none()
    {
        if element_document.body().map(|e| e.into()) != Some(element.clone())
            && computed_style
                .as_ref()
                .map(|c| c.get_property_value("position").unwrap_throw())
                != Some("fixed".into())
        {
            return true;
        }
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

        if element.parent_element().is_none()
            || element
                .parent_element()
                .unwrap_throw()
                .dyn_into::<HtmlElement>()
                .unwrap_throw()
                .offset_parent()
                .is_none()
        {
            if element_document.body().map(|b| b.into()) != element.parent_element() {
                return true;
            }
        }
    }

    false
}

pub fn is_radio(element: &Element) -> bool {
    if element.tag_name() == "INPUT" {
        let element: HtmlInputElement = element.clone().dyn_into().unwrap_throw();
        if !element.name().is_empty() && element.type_() == "radio" {
            return true;
        }
    }
    false
}

pub fn get_radio_button_group(element: &Element) -> Option<types::RadioButtonGroup> {
    if !is_radio(element) {
        return None;
    }
    let element = element
        .clone()
        .dyn_into::<HtmlInputElement>()
        .unwrap_throw();
    let name = element.name();
    let radio_buttons = DOM::get_elements_by_name(&element, &name);
    let mut checked: Option<HtmlInputElement> = None;
    let buttons = web_sys::js_sys::Set::new(&JsValue::undefined());

    for i in 0..radio_buttons.length() {
        let el = radio_buttons.item(i).unwrap_throw();
        let el: HtmlInputElement = el.dyn_into().unwrap_throw();
        if is_radio(&el) {
            if el.checked() {
                checked = Some(el.clone());
            }
            buttons.add(&el);
        }
    }

    Some(types::RadioButtonGroup {
        name,
        buttons,
        checked,
    })
}

/// If the passed element is Tabster dummy input, returns the container element this dummy input belongs to.
/// element: Element to check for being dummy input.
/// returns: Dummy input container element (if the passed element is a dummy input) or null.
pub fn get_dummy_input_container(element: &Option<Element>) -> Option<HtmlElement> {
    // return (
    //     (
    //         element as HTMLElementWithDummyContainer | null | undefined
    //     )?.__tabsterDummyContainer?.get() || null
    // );
    None
}
