use crate::{
    tabster::TabsterCore,
    types::{self, GetWindow},
    utils::{DummyInputManager, TabsterPart},
};
use std::{cell::RefCell, sync::Arc};
use web_sys::HtmlElement;

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
    part: TabsterPart,
    dummy_manager: Option<GroupperDummyManager>,
}

impl Groupper {
    pub fn new(
        tabster: Arc<RefCell<TabsterCore>>,
        element: &HtmlElement,
        sys: Option<types::SysProps>,
    ) -> Self {
        let mut this = Self {
            part: TabsterPart::new(),
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
}

pub struct GroupperAPI {
    tabster: Arc<RefCell<TabsterCore>>,
    win: Arc<GetWindow>,
}

impl GroupperAPI {
    pub fn new(tabster: Arc<RefCell<TabsterCore>>, get_window: Arc<GetWindow>) -> Self {
        Self {
            tabster,
            win: get_window,
        }
    }

    pub fn create_groupper(&self, element: &HtmlElement, sys: Option<types::SysProps>) -> Groupper {
        let new_groupper = Groupper::new(
            self.tabster.clone(),
            element,
            // this._onGroupperDispose,
            // props,
            sys,
        );

        // this._grouppers[new_groupper.id] = new_groupper;

        // const focusedElement = this._tabster.focusedElement.getFocusedElement();

        // // Newly created groupper contains currently focused element, update the state on the next tick (to
        // // make sure all grouppers are processed).
        // if (
        //     focusedElement &&
        //     dom.nodeContains(element, focusedElement) &&
        //     !this._updateTimer
        // ) {
        //     this._updateTimer = this._win().setTimeout(() => {
        //         delete this._updateTimer;
        //         // Making sure the focused element hasn't changed.
        //         if (
        //             focusedElement ===
        //             this._tabster.focusedElement.getFocusedElement()
        //         ) {
        //             this._updateCurrent(focusedElement, true, true);
        //         }
        //     }, 0);
        // }

        new_groupper
    }
}
