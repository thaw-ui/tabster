use crate::{
    dom_api::DOM,
    instance::get_tabster_on_element,
    tabster::TabsterCore,
    types::{self, FindFirstProps, GetWindow, DOMAPI},
    utils::{DummyInputManager, TabsterPart},
    web::set_timeout,
};
use std::{cell::RefCell, collections::HashMap, sync::Arc};
use web_sys::{
    wasm_bindgen::{JsCast, UnwrapThrowExt},
    HtmlElement,
};

struct GroupperDummyManager(DummyInputManager);

impl GroupperDummyManager {
    fn new(
        element: HtmlElement,
        groupper: &Groupper,
        tabster: Arc<RefCell<TabsterCore>>,
        sys: Option<types::SysProps>,
    ) -> Self {
        Self(DummyInputManager::new(tabster, element, sys))
    }
}

pub struct Groupper {
    part: TabsterPart<types::GroupperProps>,
    should_tab_inside: bool,
    first: Option<HtmlElement>,
    dummy_manager: Option<GroupperDummyManager>,
}

impl Groupper {
    pub fn new(
        tabster: Arc<RefCell<TabsterCore>>,
        element: &HtmlElement,
        props: types::GroupperProps,
        sys: Option<types::SysProps>,
    ) -> Self {
        let mut this = Self {
            part: TabsterPart::new(tabster.clone(), element.clone(), props),
            should_tab_inside: false,
            first: None,
            dummy_manager: None,
        };

        let control_tab = {
            let tabster = tabster.borrow();
            tabster.control_tab
        };
        this.dummy_manager = if !control_tab {
            Some(GroupperDummyManager::new(
                element.clone(),
                &this,
                tabster,
                sys,
            ))
        } else {
            None
        };

        this
    }

    pub fn id(&self) -> &String {
        &self.part.id
    }

    pub fn is_active(&mut self, no_if_first_is_focused: Option<bool>) -> Option<bool> {
        let element = self.part.get_element();
        let mut is_parent_active = true;

        let mut el = element.clone();
        loop {
            let Some(e) = DOM::get_parent_element(el) else {
                break;
            };
            if let Some(tabster_on_element) = get_tabster_on_element(self.part.tabster.clone(), &e)
            {
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
        let groupper_element = self.part.get_element();
        let mut first = None::<HtmlElement>;

        if let Some(groupper_element) = self.part.get_element() {
            let focusable = {
                let tabster = self.part.tabster.borrow();
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
                        // useActiveModalizer: true,
                    },
                    Default::default(),
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
        let id = new_groupper.id().clone();
        let new_groupper = Arc::new(RefCell::new(new_groupper));

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
                Some(focused_element.dyn_into().unwrap_throw()),
            ) {
                let update_timer_is_none = {
                    let update_timer = self.update_timer.borrow();
                    update_timer.is_none()
                };
                if update_timer_is_none {
                    let update_timer = self.update_timer.clone();
                    let mut update_timer_ref = self.update_timer.try_borrow_mut().unwrap_throw();
                    let timer = set_timeout(
                        &(self.win)(),
                        move || {
                            let mut update_timer = update_timer.try_borrow_mut().unwrap_throw();
                            *update_timer = None;
                            // Making sure the focused element hasn't changed.
                            // if (
                            //     focusedElement ===
                            //     this._tabster.focusedElement.getFocusedElement()
                            // ) {
                            //     this._updateCurrent(focusedElement, true, true);
                            // }
                        },
                        0,
                    );
                }
            }
        }

        new_groupper
    }
}
