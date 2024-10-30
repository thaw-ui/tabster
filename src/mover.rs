use crate::{
    tabster::TabsterCore,
    types::{self, GetWindow},
    utils::{DummyInputManager, TabsterPart},
};
use std::{cell::RefCell, collections::HashMap, sync::Arc};
use web_sys::HtmlElement;

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

pub struct Mover {
    part: TabsterPart<types::MoverProps>,
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

        if props.track_state.unwrap_or_default() || props.visibility_aware.unwrap_or_default() > 0 {
        //     this._intersectionObserver = new IntersectionObserver(
        //         this._onIntersection,
        //         { threshold: [0, 0.25, 0.5, 0.75, 1] }
        //     );
        //     this._observeState();
        }

        // this._onDispose = onDispose;
        // const getMemorized = () =>
        //     props.memorizeCurrent ? this._current : undefined;

        let mut this = Self {
            part: TabsterPart::new(tabster.clone(), element.clone(), props),
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

    pub fn id(&self) -> &String {
        &self.part.id
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
