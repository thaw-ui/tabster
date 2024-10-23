use crate::{
    tabster::{self, TabsterCore},
    types::{self, GetWindow},
    utils::DummyInputManager,
};
use std::{
    cell::RefCell,
    sync::{Arc, OnceLock, RwLock},
};
use web_sys::{wasm_bindgen::UnwrapThrowExt, HtmlElement};

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

static LAST_TABSTER_PART_ID: OnceLock<RwLock<usize>> = OnceLock::new();

pub struct Groupper {
    id: String,
    dummy_manager: Option<GroupperDummyManager>,
}

impl Groupper {
    pub fn new(
        tabster: Arc<RefCell<TabsterCore>>,
        element: &HtmlElement,
        sys: Option<types::SysProps>,
    ) -> Self {
        let last_tabster_part_id = LAST_TABSTER_PART_ID.get_or_init(Default::default);
        let id = *last_tabster_part_id.read().unwrap_throw() + 1;
        *last_tabster_part_id.write().unwrap_throw() = id;

        let id = format!("i{}", id);
        let mut this = Self {
            id,
            dummy_manager: None,
        };

        let control_tab = {
            let tabster = tabster.borrow();
            tabster.control_tab
        };
        let dummy_manager = if !control_tab {
            //     this.dummyManager = new GroupperDummyManager(
            //         this._element,
            //         this,
            //         tabster,
            //         sys
            //     );
            Some(GroupperDummyManager::new(
                element.clone(),
                &this,
                tabster,
                sys,
            ))
        } else {
            None
        };

        this.dummy_manager = dummy_manager;

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
