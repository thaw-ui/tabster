use crate::{
    console_log,
    dom_api::DOM,
    instance::get_tabster_on_element,
    state::focused_element::FOCUSED_ELEMENT_STATE_IS_TABBING,
    tabster::TabsterCore,
    types::{self, GetWindow, MoverProps, DOMAPI},
    utils::{
        get_dummy_input_container, DummyInputManager, NodeFilterEnum, TabsterPart, WeakHTMLElement,
    },
    web::{add_event_listener_with_bool, set_timeout, EventListenerHandle},
};
use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::{atomic::Ordering, Arc},
};
use web_sys::{
    wasm_bindgen::{prelude::Closure, JsCast, UnwrapThrowExt},
    Element, HtmlElement, IntersectionObserver, IntersectionObserverEntry,
    IntersectionObserverInit,
};

struct MoverDummyManager(DummyInputManager);

impl MoverDummyManager {
    fn new(
        element: WeakHTMLElement,
        tabster: Arc<RefCell<TabsterCore>>,
        sys: Option<types::SysProps>,
    ) -> Self {
        Self(DummyInputManager::new(tabster, element, sys, None))
    }
}

pub type ArcCellMover = Arc<RefCell<Mover>>;

pub struct Mover {
    part: TabsterPart<types::MoverProps>,

    win: Arc<GetWindow>,
    intersection_observer: Option<IntersectionObserver>,
    set_current_timer: Option<i32>,
    current: Option<RefCell<WeakHTMLElement>>,

    dummy_manager: Option<MoverDummyManager>,
    visibility_tolerance: f32,
}

impl Deref for Mover {
    type Target = TabsterPart<types::MoverProps>;

    fn deref(&self) -> &Self::Target {
        &self.part
    }
}

impl DerefMut for Mover {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.part
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

        let win = { tabster.borrow().get_window.clone() };

        Self {
            part: TabsterPart::new(tabster.clone(), element.clone(), props.clone()),
            win,
            intersection_observer: None,
            dummy_manager: None,
            visibility_tolerance,
            set_current_timer: None,
            current: None,
        }
        .init(tabster, props, sys)
    }

    fn init(
        mut self,
        tabster: Arc<RefCell<TabsterCore>>,
        props: types::MoverProps,
        sys: Option<types::SysProps>,
    ) -> Self {
        let control_tab = {
            let tabster = tabster.borrow();
            tabster.control_tab
        };

        self.dummy_manager = if !control_tab {
            Some(MoverDummyManager::new(
                self._element.clone(),
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
        if !FOCUSED_ELEMENT_STATE_IS_TABBING.load(Ordering::SeqCst) {
            let Some(current_ctx) = state.current_ctx.as_ref() else {
                return None;
            };

            if current_ctx.excluded_from_mover.unwrap() {
                return Some(*NodeFilterEnum::FilterReject);
            } else {
                return None;
            }
        }

        let MoverProps {
            memorize_current,
            visibility_aware,
            has_default,
            ..
        } = self.props.clone();

        let has_default = has_default.unwrap_or(true);
        let mover_element = self.get_element();

        if mover_element.is_some()
            && (memorize_current.unwrap_or_default()
                || visibility_aware.unwrap_or_default() != 0
                || has_default)
            && (!DOM::node_contains(
                mover_element.clone().map(Into::into),
                Some(state.from.clone().into()),
            ) || get_dummy_input_container(&Some(state.from.clone().into())) == mover_element)
        {
            let mut found: Option<HtmlElement> = None;

            if memorize_current.unwrap_or_default() {
                let current = if let Some(current) = &self.current {
                    current.borrow_mut().get()
                } else {
                    None
                };

                if let Some(current) = current {
                    if (state.accept_condition)(current.clone()) {
                        found = Some(current);
                    }
                }
            }

            if found.is_none() && has_default {
                found = self
                    .tabster
                    .borrow()
                    .focusable
                    .clone()
                    .map(|f| {
                        f.borrow_mut().find_default(
                            types::FindDefaultProps {
                                container: mover_element.clone().unwrap_throw(),
                                modalizer_id: None,
                                include_programmatically_focusable: None,
                                use_active_modalizer: Some(true),
                                ignore_accessibility: None,
                            },
                            &mut types::FindFocusableOutputProps::default(),
                        )
                    })
                    .flatten();
            }

            if found.is_none() && visibility_aware.unwrap_or_default() != 0 {
                found = self
                    .tabster
                    .borrow()
                    .focusable
                    .clone()
                    .map(|f| {
                        f.borrow_mut().find_element(
                            types::FindFocusableProps {
                                container: mover_element.clone().unwrap_throw(),
                                current_element: None,
                                reference_element: None,
                                include_programmatically_focusable: None,
                                ignore_accessibility: None,
                                use_active_modalizer: Some(true),
                                modalizer_id: None,
                                is_backward: state.is_backward,
                                accept_condition: Some(Box::new(move |el| {
                                    // const id = getElementUId(this._win, el);
                                    // const visibility = this._visible[id];
                                    // return (
                                    //     moverElement !== el &&
                                    //     !!this._allElements?.get(el) &&
                                    //     state.acceptCondition(el) &&
                                    //     (visibility === Visibilities.Visible ||
                                    //         (visibility === Visibilities.PartiallyVisible &&
                                    //             (visibilityAware ===
                                    //                 Visibilities.PartiallyVisible ||
                                    //                 !this._fullyVisible)))
                                    // );
                                    false
                                })),
                                on_element: None,
                            },
                            &mut types::FindFocusableOutputProps::default(),
                        )
                    })
                    .flatten();
            }

            if let Some(found) = found {
                state.found = Some(true);
                state.found_element = Some(found);
                state.reject_elements_from = mover_element;
                state.skipped_focusable = Some(true);
                return Some(*NodeFilterEnum::FilterAccept);
            }
        }

        None
    }

    fn observe_state(&self) {
        let element = self.get_element();

        if element.is_none() {
            return;
        }
    }

    fn set_current(&mut self, element: Option<HtmlElement>) {
        if let Some(element) = element {
            self.current = Some(WeakHTMLElement::new(self.win.clone(), element, None).into());
        } else {
            self.current = None;
        }

        if self.props.track_state.unwrap_or_default()
            || self.props.visibility_aware.unwrap_or_default() != 0
        {
            self.set_current_timer = set_timeout(
                &(self.win)(),
                move || {
                    // TODO
                },
                0,
            )
            .into();
        }
    }
}

pub struct MoverAPI {
    tabster: Arc<RefCell<TabsterCore>>,
    win: Arc<GetWindow>,
    movers: HashMap<String, Arc<RefCell<Mover>>>,
    event_listener_handle_keydown: Arc<RefCell<Option<EventListenerHandle>>>,
}

impl MoverAPI {
    pub fn new(tabster: Arc<RefCell<TabsterCore>>, get_window: Arc<GetWindow>) -> Self {
        let event_listener_handle_keydown: Arc<RefCell<Option<EventListenerHandle>>> =
            Default::default();

        tabster.borrow_mut().queue_init({
            let tabster = tabster.clone();
            let get_window = get_window.clone();
            let event_listener_handle_keydown = event_listener_handle_keydown.clone();
            move || {
                let win = get_window();

                *event_listener_handle_keydown.borrow_mut() = Some(add_event_listener_with_bool(
                    win,
                    "keydown",
                    move |_: web_sys::KeyboardEvent| {
                        // this._onKeyDown
                    },
                    true,
                ));
                // win.addEventListener(MoverMoveFocusEventName, this._onMoveFocus);
                // win.addEventListener(
                //     MoverMemorizedElementEventName,
                //     this._onMemorizedElement
                // );

                let on_focus = {
                    let tabster = tabster.clone();
                    move |element: HtmlElement| {
                        // When something in the app gets focused, we are making sure that
                        // the relevant context Mover is aware of it.
                        // Looking for the relevant context Mover from the currently
                        // focused element parent, not from the element itself, because the
                        // Mover element itself cannot be its own current (but might be
                        // current for its parent Mover).
                        let mut current_focusable_element = Some(element.clone());
                        let mut deepest_focusable_element = element.clone();

                        let mut el = DOM::get_parent_element(Some(element));
                        while let Some(new_el) = el {
                            // We go through all Movers up from the focused element and
                            // set their current element to the deepest focusable of that
                            // Mover.
                            let mover = get_tabster_on_element(&tabster, &new_el)
                                .map(|value| value.borrow().mover.clone())
                                .flatten();

                            if let Some(mover) = mover {
                                mover
                                    .borrow_mut()
                                    .set_current(Some(deepest_focusable_element.clone()));
                                current_focusable_element = None;
                            }

                            if current_focusable_element.is_none()
                                && tabster.borrow().focusable.clone().is_some_and(|f| {
                                    f.borrow().is_focusable(&new_el, None, None, None)
                                })
                            {
                                deepest_focusable_element = new_el.clone();
                                current_focusable_element = Some(new_el.clone());
                            }

                            el = DOM::get_parent_element(Some(new_el));
                        }
                    }
                };

                if let Some(focused_element) = tabster.borrow_mut().focused_element.as_mut() {
                    focused_element.subscribe(on_focus);
                }
            }
        });
        Self {
            tabster,
            win: get_window,
            movers: HashMap::new(),
            event_listener_handle_keydown,
        }
    }

    fn dispose(self) {
        if let Some(handle) = self.event_listener_handle_keydown.borrow_mut().take() {
            handle.remove();
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
