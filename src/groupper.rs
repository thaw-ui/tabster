use crate::{
    dom_api::DOM,
    instance::get_tabster_on_element,
    keyborg::native_focus,
    root::RootAPI,
    state::focused_element::FocusedElementState,
    tabster::TabsterCore,
    types::{self, CachedGroupper, FindFirstProps, GetWindow, DOMAPI},
    utils::{
        get_adjacent_element, get_dummy_input_container, DummyInputManager, NodeFilterEnum,
        TabsterPart,
    },
    web::set_timeout,
    GroupperTabbabilities,
};
use std::{
    cell::{RefCell, RefMut},
    collections::{HashMap, HashSet},
    ops::Deref,
    sync::Arc,
};
use web_sys::{
    wasm_bindgen::{JsCast, UnwrapThrowExt},
    Element, HtmlElement,
};

struct GroupperDummyManager(DummyInputManager);

impl GroupperDummyManager {
    fn new(
        element: HtmlElement,
        groupper: Arc<RefCell<Groupper>>,
        tabster: Arc<RefCell<TabsterCore>>,
        sys: Option<types::SysProps>,
    ) -> Self {
        let mut dummy_input_manager =
            DummyInputManager::new(tabster.clone(), element.clone(), sys, Some(true));
        dummy_input_manager.set_handlers(
            Some(Box::new(move |dummy_input, is_backward, related_target| {
                let container = element.clone();
                if let Some(input) = dummy_input.input {
                    if let Some(ctx) =
                        RootAPI::get_tabster_context(&tabster, &input, Default::default())
                    {
                        let mut groupper = groupper.borrow_mut();
                        let mut next = if let Some(next_tabbable) = groupper.find_next_tabbable(
                            related_target,
                            None,
                            Some(is_backward),
                            Some(true),
                        ) {
                            next_tabbable.element
                        } else {
                            None
                        };

                        if next.is_none() {
                            let current_element = if dummy_input.is_outside {
                                Some(input)
                            } else {
                                get_adjacent_element(container, Some(!is_backward))
                            };
                            let next_tabbable = FocusedElementState::find_next_tabbable(
                                &tabster,
                                ctx,
                                None,
                                current_element,
                                None,
                                Some(is_backward),
                                Some(true),
                            );

                            next = next_tabbable.map(|n| n.element).flatten()
                        }

                        if let Some(next) = next {
                            native_focus(next);
                        }
                    }
                }
            })),
            None,
        );
        Self(dummy_input_manager)
    }
}

pub struct Groupper {
    part: TabsterPart<types::GroupperProps>,
    should_tab_inside: bool,
    first: Option<HtmlElement>,
    dummy_manager: Option<GroupperDummyManager>,
}

impl Deref for Groupper {
    type Target = TabsterPart<types::GroupperProps>;

    fn deref(&self) -> &Self::Target {
        &self.part
    }
}

impl Groupper {
    pub fn new(
        tabster: Arc<RefCell<TabsterCore>>,
        element: &HtmlElement,
        props: types::GroupperProps,
        sys: Option<types::SysProps>,
    ) -> Arc<RefCell<Self>> {
        let this = Arc::new(RefCell::new(Self {
            part: TabsterPart::new(tabster.clone(), element.clone(), props),
            should_tab_inside: false,
            first: None,
            dummy_manager: None,
        }));

        let control_tab = {
            let tabster = tabster.borrow();
            tabster.control_tab
        };
        let dummy_manager = if !control_tab {
            Some(GroupperDummyManager::new(
                element.clone(),
                this.clone(),
                tabster,
                sys,
            ))
        } else {
            None
        };
        {
            let mut this = this.borrow_mut();
            this.dummy_manager = dummy_manager;
        }
        this
    }

    fn find_next_tabbable(
        &mut self,
        current_element: Option<HtmlElement>,
        reference_element: Option<HtmlElement>,
        is_backward: Option<bool>,
        ignore_accessibility: Option<bool>,
    ) -> Option<types::NextTabbable> {
        let Some(groupper_element) = self.get_element() else {
            return None;
        };

        let current_is_dummy =
            get_dummy_input_container(&current_element.clone().map(|e| e.into())).as_ref()
                == Some(&groupper_element);

        if !self.should_tab_inside
            && current_element.is_some()
            && DOM::node_contains(
                Some(groupper_element.clone().into()),
                Some(current_element.clone().unwrap_throw().into()),
            )
            && !current_is_dummy
        {
            return Some(types::NextTabbable {
                element: None,
                uncontrolled: None,
                out_of_dom_order: Some(true),
            });
        }

        let groupper_first_focusable = self.get_first(true);
        if current_element.is_none()
            || DOM::node_contains(
                Some(groupper_element.clone().into()),
                Some(current_element.clone().unwrap_throw().into()),
            )
            || current_is_dummy
        {
            return Some(types::NextTabbable {
                element: groupper_first_focusable,
                uncontrolled: None,
                out_of_dom_order: Some(true),
            });
        }

        // const tabster = this._tabster;
        let mut next = None::<HtmlElement>;
        let mut out_of_dom_order = false;
        let mut uncontrolled = None::<HtmlElement>;

        if self.should_tab_inside && groupper_first_focusable.is_some() {
            let find_props = types::FindNextProps {
                current_element,
                reference_element,
                container: groupper_element.clone(),
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

            if next.is_none()
                && self.props.tabbability == Some(*GroupperTabbabilities::LimitedTrapFocus)
            {
                let find_props = types::FindFirstProps {
                    container: groupper_element,
                    ignore_accessibility,
                    use_active_modalizer: Some(true),
                };

                next = if is_backward.unwrap_or_default() {
                    focusable.find_last(find_props, &mut find_props_out)
                } else {
                    focusable.find_first(find_props, &mut find_props_out)
                };

                out_of_dom_order = true;
            }

            uncontrolled = find_props_out.uncontrolled;
        }

        Some(types::NextTabbable {
            element: next,
            uncontrolled,
            out_of_dom_order: Some(out_of_dom_order),
        })
    }

    pub fn is_active(&mut self, no_if_first_is_focused: Option<bool>) -> Option<bool> {
        let element = self.part.get_element();
        let mut is_parent_active = true;

        let mut el = element.clone();
        loop {
            let Some(e) = DOM::get_parent_element(el) else {
                break;
            };
            if let Some(tabster_on_element) = get_tabster_on_element(&self.tabster, &e) {
                let tabster_on_element = tabster_on_element.borrow();
                if let Some(g) = tabster_on_element.groupper.clone() {
                    let g = g.borrow();
                    if !g.should_tab_inside {
                        is_parent_active = false;
                    }
                }
            }
            el = Some(e);
        }

        let mut ret = if is_parent_active {
            if self.part.props.tabbability.unwrap_or_default() > 0 {
                Some(self.should_tab_inside)
            } else {
                Some(false)
            }
        } else {
            None
        };

        if ret.unwrap_or_default() && no_if_first_is_focused.unwrap_or_default() {
            let focused = {
                let tabster = self.part.tabster.borrow();
                tabster
                    .focused_element
                    .as_ref()
                    .unwrap_throw()
                    .get_focused_element()
            };

            ret = Some(focused != self.get_first(true));
        }

        ret
    }

    fn get_first(&mut self, or_container: bool) -> Option<HtmlElement> {
        let mut first = None::<HtmlElement>;

        if let Some(groupper_element) = self.get_element() {
            let focusable = {
                let tabster = self.tabster.borrow();
                tabster.focusable.clone().unwrap_throw()
            };
            let mut focusable = focusable.borrow_mut();

            if or_container && focusable.is_focusable(&groupper_element, None, None, None) {
                return Some(groupper_element);
            }

            first = self.first.clone();

            if first.is_none() {
                first = focusable.find_first(
                    FindFirstProps {
                        container: groupper_element,
                        ignore_accessibility: None,
                        use_active_modalizer: Some(true),
                    },
                    &mut Default::default(),
                );

                if first.is_some() {
                    self.set_first(first.clone());
                }
            }
        }

        first
    }

    fn set_first(&mut self, element: Option<HtmlElement>) {
        if let Some(element) = element {
            self.first = Some(element);
        } else {
            self.first = None;
        }
    }

    fn get_is_active(
        &mut self,
        state: &mut RefMut<'_, types::FocusableAcceptElementState>,
        groupper_id: &str,
    ) -> Option<bool> {
        let cached = state.cached_grouppers.get(groupper_id);
        let is_active: Option<bool>;

        if let Some(cached) = cached {
            is_active = cached.is_active;
        } else {
            is_active = self.is_active(Some(true));

            state.cached_grouppers.insert(
                groupper_id.to_string(),
                CachedGroupper {
                    is_active,
                    first: None,
                },
            );
        }

        is_active
    }

    pub fn accept_element(
        &mut self,
        element: &Element,
        state: &mut RefMut<'_, types::FocusableAcceptElementState>,
    ) -> Option<u32> {
        let parent_element = DOM::get_parent_element(self.get_element());
        let parent_ctx = if let Some(parent_element) = &parent_element {
            RootAPI::get_tabster_context(&self.tabster, &parent_element, Default::default())
        } else {
            None
        };

        let mut parent_ctx_groupper = None;
        let mut parent_groupper = None;
        if let Some(parent_ctx) = &parent_ctx {
            parent_ctx_groupper = parent_ctx.groupper.clone();
            if parent_ctx.groupper_before_mover.unwrap_or_default() {
                parent_groupper = parent_ctx_groupper.clone();
            }
        }

        let mut parent_groupper_element = None::<HtmlElement>;

        if let Some(parent_groupper) = &parent_groupper {
            let parent_groupper = parent_groupper.borrow();
            parent_groupper_element = parent_groupper.get_element();

            let is_active = { self.get_is_active(state, &parent_groupper.id) };
            if !is_active.unwrap_or_default()
                && parent_groupper_element.is_some()
                && Some(state.container.clone()) != parent_groupper_element
                && DOM::node_contains(
                    Some(state.container.clone().into()),
                    parent_groupper_element.clone().map(|el| el.into()),
                )
            {
                // Do not fall into a child groupper of inactive parent groupper if it's in the scope of the search.
                state.skipped_focusable = Some(true);
                return Some(*NodeFilterEnum::FilterReject);
            }
        }

        let is_active = self.get_is_active(state, &self.id.clone());
        let groupper_element = self.get_element();

        if let Some(groupper_element) = groupper_element {
            if is_active.unwrap_or_default() != true {
                // if &groupper_element.clone().into() == element {
                //     if let Some(parent_ctx_groupper) = &parent_ctx_groupper {
                //         let parent_ctx_groupper = parent_ctx_groupper.borrow();

                //         if parent_groupper_element.is_none() {
                //             parent_groupper_element = parent_ctx_groupper.get_element();
                //         }

                // if
                //     parent_groupper_element.is_some() &&
                //     !self.get_is_active(state, &parent_ctx_groupper.id).unwrap_or_default() &&
                //     DOM::node_contains(
                //         state.container,
                //         parentGroupperElement
                //     ) &&
                //     parentGroupperElement !== state.container
                // {
                //     state.skipped_focusable = Some(true);
                //     return Some(*NodeFilterEnum::FilterReject);
                // }
                //     }
                // }

                //         if (
                //             groupperElement !== element &&
                //             dom.nodeContains(groupperElement, element)
                //         ) {
                //             state.skippedFocusable = true;
                //             return NodeFilter.FILTER_REJECT;
                //         }

                //         const cached = cachedGrouppers[this.id];
                //         let first: HTMLElement | null | undefined;

                //         if ("first" in cached) {
                //             first = cached.first;
                //         } else {
                //             first = cached.first = this.getFirst(true);
                //         }

                //         if (first && state.acceptCondition(first)) {
                //             state.rejectElementsFrom = groupperElement;
                //             state.skippedFocusable = true;

                //             if (first !== state.from) {
                //                 state.found = true;
                //                 state.foundElement = first;
                //                 return NodeFilter.FILTER_ACCEPT;
                //             } else {
                //                 return NodeFilter.FILTER_REJECT;
                //             }
                //         }
            }
        }
        None
    }
}

pub struct GroupperAPI {
    tabster: Arc<RefCell<TabsterCore>>,
    update_timer: Arc<RefCell<Option<i32>>>,
    win: Arc<GetWindow>,
    grouppers: HashMap<String, Arc<RefCell<Groupper>>>,
}

impl GroupperAPI {
    pub fn new(tabster: Arc<RefCell<TabsterCore>>, get_window: Arc<GetWindow>) -> Self {
        Self {
            tabster,
            update_timer: Default::default(),
            win: get_window,
            grouppers: HashMap::new(),
        }
    }

    pub fn create_groupper(
        &mut self,
        element: &HtmlElement,
        props: types::GroupperProps,
        sys: Option<types::SysProps>,
    ) -> Arc<RefCell<Groupper>> {
        let new_groupper = Groupper::new(
            self.tabster.clone(),
            element,
            // this._onGroupperDispose,
            props,
            sys,
        );
        let id = {
            let new_groupper = new_groupper.borrow();
            new_groupper.id().clone()
        };

        self.grouppers.insert(id, new_groupper.clone());

        let focused_element = {
            let tabster = self.tabster.borrow();
            let focused_element = tabster.focused_element.as_ref().unwrap_throw();
            focused_element.get_focused_element()
        };

        // Newly created groupper contains currently focused element, update the state on the next tick (to
        // make sure all grouppers are processed).
        if let Some(focused_element) = focused_element {
            if DOM::node_contains(
                Some(element.clone().dyn_into().unwrap_throw()),
                Some(focused_element.clone().dyn_into().unwrap_throw()),
            ) {
                let update_timer_is_none = {
                    let update_timer = self.update_timer.borrow();
                    update_timer.is_none()
                };
                if update_timer_is_none {
                    let update_timer = self.update_timer.clone();
                    let mut update_timer_ref = self.update_timer.try_borrow_mut().unwrap_throw();
                    let tabster = self.tabster.clone();
                    let timer = set_timeout(
                        &(self.win)(),
                        move || {
                            let mut update_timer = update_timer.try_borrow_mut().unwrap_throw();
                            *update_timer = None;
                            let fe = {
                                let tabster = tabster.borrow();
                                if let Some(fe) = tabster.focused_element.as_ref() {
                                    fe.get_focused_element()
                                } else {
                                    None
                                }
                            };
                            // Making sure the focused element hasn't changed.
                            if Some(&focused_element) == fe.as_ref() {
                                Self::update_current(
                                    tabster.clone(),
                                    &focused_element,
                                    Some(true),
                                    Some(true),
                                );
                            }
                        },
                        0,
                    );
                    *update_timer_ref = Some(timer);
                }
            }
        }

        new_groupper
    }

    fn update_current(
        tabster: Arc<RefCell<TabsterCore>>,
        element: &HtmlElement,
        include_target: Option<bool>,
        check_target: Option<bool>,
    ) {
        // if (this._updateTimer) {
        //     this._win().clearTimeout(this._updateTimer);
        //     delete this._updateTimer;
        // }

        let mut new_ids = HashSet::new();
        let mut is_target = true;
        let mut el = Some(element.clone());

        loop {
            let Some(el_ref) = el.as_ref() else {
                break;
            };
            if let Some(tabster_on_element) = get_tabster_on_element(&tabster, el_ref) {
                let tabster_on_element = tabster_on_element.borrow();
                if let Some(groupper) = tabster_on_element.groupper.as_ref() {
                    let groupper = groupper.borrow();
                    new_ids.insert(groupper.id.clone());

                    if is_target && check_target.unwrap_or_default() && el_ref != element {
                        is_target = false;
                    }

                    if include_target.unwrap_or_default() || !is_target {
                        //             this._current[groupper.id] = groupper;
                        //             const isTabbable =
                        //                 groupper.isActive() ||
                        //                 (element !== el &&
                        //                     (!groupper.getProps().delegated ||
                        //                         groupper.getFirst(false) !== element));

                        //             groupper.makeTabbable(isTabbable);
                    }

                    is_target = false;
                }
            }
            el = DOM::get_parent_element(Some(el_ref.clone()));
        }

        // for (const id of Object.keys(this._current)) {
        //     const groupper = this._current[id];

        //     if (!(groupper.id in newIds)) {
        //         groupper.makeTabbable(false);
        //         groupper.setFirst(undefined);
        //         delete this._current[id];
        //     }
        // }
    }
}
