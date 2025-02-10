use crate::{
    console_log,
    dom_api::DOM,
    tabster::TabsterCore,
    types::{self, GetWindow, DOMAPI},
    utils::{get_dummy_input_container, DummyInputManager, TabsterPart},
};
use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
    ops::Deref,
    sync::Arc,
};
use web_sys::{
    wasm_bindgen::{prelude::Closure, JsCast, UnwrapThrowExt},
    Element, HtmlElement, IntersectionObserver, IntersectionObserverEntry,
    IntersectionObserverInit,
};

struct MoverDummyManager(DummyInputManager);

impl MoverDummyManager {
    fn new(
        element: HtmlElement,
        tabster: Arc<RefCell<TabsterCore>>,
        sys: Option<types::SysProps>,
    ) -> Self {
        Self(DummyInputManager::new(tabster, element, sys, None))
    }
}

pub type ArcCellMover = Arc<RefCell<Mover>>;

pub struct Mover {
    part: TabsterPart<types::MoverProps>,

    intersection_observer: Option<IntersectionObserver>,

    dummy_manager: Option<MoverDummyManager>,
    visibility_tolerance: f32,
}

impl Deref for Mover {
    type Target = TabsterPart<types::MoverProps>;

    fn deref(&self) -> &Self::Target {
        &self.part
    }
}

impl Mover {
    pub fn new(
        tabster: Arc<RefCell<TabsterCore>>,
        element: &HtmlElement,
        props: types::MoverProps,
        sys: Option<types::SysProps>,
    ) -> Self {
        console_log!("Mover::new");

        // this._win = tabster.getWindow;
        let visibility_tolerance = props.visibility_tolerance.unwrap_or(0.8);

        // this._onDispose = onDispose;
        // const getMemorized = () =>
        //     props.memorizeCurrent ? this._current : undefined;

        Self {
            part: TabsterPart::new(tabster.clone(), element.clone(), props.clone()),
            intersection_observer: None,
            dummy_manager: None,
            visibility_tolerance,
        }
        .init(tabster, element, props, sys)
    }

    fn init(
        mut self,
        tabster: Arc<RefCell<TabsterCore>>,
        element: &HtmlElement,
        props: types::MoverProps,
        sys: Option<types::SysProps>,
    ) -> Self {
        let control_tab = {
            let tabster = tabster.borrow();
            tabster.control_tab
        };

        self.dummy_manager = if !control_tab {
            Some(MoverDummyManager::new(
                element.clone(),
                tabster,
                // getMemorized,
                sys,
            ))
        } else {
            None
        };

        if props.track_state.unwrap_or_default() || props.visibility_aware.unwrap_or_default() > 0 {
            let on_intersection: Closure<dyn Fn(Vec<IntersectionObserverEntry>)> = Closure::new(
                move |entries: Vec<IntersectionObserverEntry>| {
                    for entry in entries.into_iter() {}
                },
            );
            let on_intersection = on_intersection.into_js_value();
            let options = IntersectionObserverInit::new();
            let threshold =
                serde_wasm_bindgen::to_value(&[0.0, 0.25, 0.5, 0.75, 1.0]).unwrap_throw();
            options.set_threshold(&threshold);
            self.intersection_observer = Some(
                IntersectionObserver::new_with_options(
                    on_intersection.as_ref().unchecked_ref(),
                    &options,
                )
                .unwrap_throw(),
            );

            self.observe_state();
        }

        self
    }

    pub fn id(&self) -> &String {
        &self.part.id
    }

    pub(crate) fn find_next_tabbable(
        &mut self,
        current_element: Option<HtmlElement>,
        reference_element: Option<HtmlElement>,
        is_backward: Option<bool>,
        ignore_accessibility: Option<bool>,
    ) -> Option<types::NextTabbable> {
        let container = self.get_element();
        let Some(container) = container else {
            return None;
        };

        let current_is_dummy = get_dummy_input_container(&current_element.clone().map(Into::into))
            == Some(container.clone());

        let mut next = None::<HtmlElement>;
        let mut out_of_dom_order = false;
        let mut uncontrolled = None::<HtmlElement>;

        if self.props.tabbable.unwrap_or_default()
            || current_is_dummy
            || current_element.clone().is_some_and(|el| {
                !DOM::node_contains(Some(container.clone().into()), Some(el.into()))
            })
        {
            let find_props = types::FindNextProps {
                current_element,
                reference_element,
                container,
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

            out_of_dom_order = find_props_out.out_of_dom_order.unwrap_throw();

            uncontrolled = find_props_out.uncontrolled;
        }

        Some(types::NextTabbable {
            element: next,
            uncontrolled,
            out_of_dom_order: Some(out_of_dom_order),
        })
    }

    pub fn accept_element(
        &self,
        element: &Element,
        state: &mut RefMut<'_, types::FocusableAcceptElementState>,
    ) -> Option<u32> {
        None
    }

    fn observe_state(&self) {
        let element = self.get_element();

        if element.is_none() {
            return;
        }
    }
}

pub struct MoverAPI {
    tabster: Arc<RefCell<TabsterCore>>,
    win: Arc<GetWindow>,
    movers: HashMap<String, Arc<RefCell<Mover>>>,
}

impl MoverAPI {
    pub fn new(tabster: Arc<RefCell<TabsterCore>>, get_window: Arc<GetWindow>) -> Self {
        // tabster.queueInit(this._init);
        Self {
            tabster,
            win: get_window,
            movers: HashMap::new(),
        }
    }

    pub fn create_mover(
        &mut self,
        element: &HtmlElement,
        props: types::MoverProps,
        sys: Option<types::SysProps>,
    ) -> Arc<RefCell<Mover>> {
        let new_mover = Mover::new(
            self.tabster.clone(),
            element,
            // this._onMoverDispose,
            props,
            sys,
        );
        let id = new_mover.id().clone();
        let new_mover = Arc::new(RefCell::new(new_mover));
        self.movers.insert(id, new_mover.clone());

        new_mover
    }
}
