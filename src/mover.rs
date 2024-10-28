use crate::{
    tabster::TabsterCore,
    types::{self, Visibility},
    utils::{DummyInputManager, TabsterPart},
};
use std::{cell::RefCell, sync::Arc};
use web_sys::HtmlElement;

struct MoverDummyManager(DummyInputManager);

impl MoverDummyManager {
    fn new(
        element: HtmlElement,
        tabster: Arc<RefCell<TabsterCore>>,
        sys: Option<types::SysProps>,
    ) -> Self {
        Self(DummyInputManager::new(tabster, element, sys))
    }
}

pub struct Mover {
    part: TabsterPart,
    dummy_manager: Option<MoverDummyManager>,
    visibility_tolerance: f32,
}

impl Mover {
    pub fn new(
        tabster: Arc<RefCell<TabsterCore>>,
        element: &HtmlElement,
        props: types::MoverProps,
        sys: Option<types::SysProps>,
    ) -> Self {
        // super(tabster, element, props);

        // this._win = tabster.getWindow;
        let visibility_tolerance = props.visibility_tolerance.unwrap_or(0.8);

        // if (this._props.trackState || this._props.visibilityAware) {
        //     this._intersectionObserver = new IntersectionObserver(
        //         this._onIntersection,
        //         { threshold: [0, 0.25, 0.5, 0.75, 1] }
        //     );
        //     this._observeState();
        // }

        // this._onDispose = onDispose;
        // const getMemorized = () =>
        //     props.memorizeCurrent ? this._current : undefined;

        let mut this = Self {
            part: TabsterPart::new(),
            dummy_manager: None,
            visibility_tolerance,
        };

        let control_tab = {
            let tabster = tabster.borrow();
            tabster.control_tab
        };
        this.dummy_manager = if !control_tab {
            Some(MoverDummyManager::new(
                element.clone(),
                tabster,
                // getMemorized,
                sys,
            ))
        } else {
            None
        };

        this
    }
}

pub struct MoverAPI {
    tabster: Arc<RefCell<TabsterCore>>,
}

impl MoverAPI {
    pub fn create_mover(
        &self,
        element: &HtmlElement,
        props: types::MoverProps,
        sys: Option<types::SysProps>,
    ) -> Mover {
        let new_mover = Mover::new(
            self.tabster.clone(),
            element,
            // this._onMoverDispose,
            props,
            sys,
        );
        // this._movers[new_mover.id] = new_mover;
        new_mover
    }
}
