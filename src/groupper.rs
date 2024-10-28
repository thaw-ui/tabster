use crate::{
    dom_api::DOM,
    state::focused_element,
    tabster::TabsterCore,
    types::{self, GetWindow, DOMAPI},
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
