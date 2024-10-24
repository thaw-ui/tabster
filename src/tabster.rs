use std::{cell::RefCell, collections::HashMap, sync::Arc};

use crate::{
    dom_api::DOM,
    focusable::FocusableAPI,
    groupper::GroupperAPI,
    mover::MoverAPI,
    root::{RootAPI, WindowWithTabsterInstance},
    types::{self, GetWindow, TabsterCoreProps, DOMAPI},
    web::set_timeout,
};
use web_sys::{
    js_sys::WeakMap,
    wasm_bindgen::{prelude::Closure, JsCast, UnwrapThrowExt},
    HtmlElement, Node, Window,
};

pub fn create_tabster(win: Window, props: TabsterCoreProps) -> Tabster {
    let tabster = TabsterCore::new(win, props);
    Tabster::new(tabster)
}

pub struct Tabster {
    pub focusable: FocusableAPI,
    pub root: RootAPI,
    pub core: Arc<RefCell<TabsterCore>>,
}

impl Tabster {
    fn new(tabster: TabsterCore) -> Tabster {
        let tabster = Arc::new(RefCell::new(tabster));
        let focusable = FocusableAPI::new(tabster.clone());
        let root = RootAPI::new(tabster.clone());

        Self {
            focusable,
            root,
            core: tabster,
        }
    }
}

// TODO Memory leak
struct TabsterCoreStorage {
    storage: web_sys::js_sys::WeakMap,
    data: HashMap<String, Arc<RefCell<types::TabsterElementStorage>>>,
}

impl TabsterCoreStorage {
    fn new() -> Self {
        Self {
            storage: WeakMap::new(),
            data: HashMap::new(),
        }
    }
    fn get_storage_value(&self, el: &HtmlElement) -> Option<String> {
        let value = self.storage.get(el);
        value.as_string()
    }
    fn get(&self, el: &HtmlElement) -> Option<Arc<RefCell<types::TabsterElementStorage>>> {
        let value = self.get_storage_value(el)?;
        self.data.get(&value).cloned()
    }

    fn set(&mut self, el: &HtmlElement, value: Arc<RefCell<types::TabsterElementStorage>>) {
        let uuid = uuid::Uuid::new_v4().to_string();
        self.storage
            .set(el, &web_sys::wasm_bindgen::JsValue::from_str(&uuid));
        self.data.insert(uuid, value);
    }

    fn delete(&mut self, el: &HtmlElement) {
        if let Some(value) = self.get_storage_value(el) {
            self.data.remove(&value);
        }
        self.storage.delete(&el);
    }
}

pub struct TabsterCore {
    storage: TabsterCoreStorage,
    win: Option<WindowWithTabsterInstance>,
    init_queue: Arc<RefCell<Vec<Box<dyn FnOnce()>>>>,
    init_timer: Arc<RefCell<Option<i32>>>,
    pub(crate) noop: bool,
    pub control_tab: bool,
    pub get_window: Arc<GetWindow>,

    // CoreAPIs
    internal: Arc<RefCell<types::InternalAPI>>,

    // Extended APIs
    pub groupper: Option<Arc<GroupperAPI>>,
    pub mover: Option<MoverAPI>,
    pub modalizer: Option<types::ModalizerAPI>,
    pub get_parent: Box<dyn Fn(Node) -> Option<Node>>,
}

impl TabsterCore {
    fn new(win: Window, props: TabsterCoreProps) -> Self {
        let get_parent = props
            .get_parent
            .unwrap_or_else(|| Box::new(move |node| DOM::get_parent_node(Some(node))));
        let internal = Arc::new(RefCell::new(types::InternalAPI::new(win.clone())));
        let get_window = {
            let win = win.clone();
            Arc::new(Box::new(move || win.clone()) as GetWindow)
        };
        let mut this = Self {
            storage: TabsterCoreStorage::new(),
            get_parent,
            win: Some(win),
            noop: false,
            control_tab: props.control_tab.unwrap_or(true),
            get_window,
            internal: internal.clone(),
            groupper: None,
            mover: None,
            modalizer: None,
            init_queue: Default::default(),
            init_timer: Default::default(),
        };

        this.queue_init(move || {
            let mut internal = internal.try_borrow_mut().unwrap_throw();
            internal.resume_observer(true);
        });

        this
    }

    fn queue_init(&mut self, callback: impl FnOnce() + 'static) {
        let Some(win) = self.win.as_ref() else {
            return;
        };

        {
            let mut init_queue = self.init_queue.try_borrow_mut().unwrap_throw();
            init_queue.push(Box::new(callback));
        }

        let init_timer_is_none = {
            let init_timer = self.init_timer.borrow();
            init_timer.is_none()
        };

        if init_timer_is_none {
            let init_timer = self.init_timer.clone();
            let mut init_timer_ref = self.init_timer.try_borrow_mut().unwrap_throw();
            let drain_init_queue_fn = self.drain_init_queue_fn();
            let timer = set_timeout(
                win,
                move || {
                    let mut init_timer = init_timer.try_borrow_mut().unwrap_throw();
                    *init_timer = None;
                    drain_init_queue_fn();
                },
                0,
            );
            *init_timer_ref = Some(timer);
        }
    }

    fn drain_init_queue_fn(&self) -> Box<dyn Fn()> {
        let init_queue = self.init_queue.clone();
        Box::new(move || {
            let mut init_queue = init_queue.try_borrow_mut().unwrap_throw();
            let queue = init_queue.drain(..);
            queue.into_iter().for_each(|callback| callback());
        })
    }

    pub fn drain_init_queue(&mut self) {
        if self.win.is_none() {
            return;
        }

        let mut init_queue = self.init_queue.try_borrow_mut().unwrap_throw();
        // Resetting the queue before calling the callbacks to avoid recursion.
        let queue = init_queue.drain(..);
        queue.into_iter().for_each(|callback| callback());
    }

    pub fn storage_entry(
        &mut self,
        element: &HtmlElement,
        addremove: Option<bool>,
    ) -> Option<Arc<RefCell<types::TabsterElementStorageEntry>>> {
        let mut entry = self.storage.get(element);
        if let Some(entry) = entry.as_ref() {
            let entry = entry.borrow();
            if matches!(addremove, Some(false)) && entry.is_empty() {
                self.storage.delete(element);
            }
        } else if matches!(addremove, Some(true)) {
            entry = Some(Arc::new(RefCell::new(
                types::TabsterElementStorageEntry::new(),
            )));
            self.storage.set(element, entry.clone().unwrap());
        }

        entry
    }
}

/// Creates a new groupper instance or returns an existing one
/// @param tabster Tabster instance
pub fn get_groupper(tabster: &Tabster) -> Arc<GroupperAPI> {
    let tabster_core = tabster.core.clone();
    let mut tabster_core_ref = tabster_core.try_borrow_mut().unwrap_throw();

    if tabster_core_ref.groupper.is_none() {
        tabster_core_ref.groupper = Some(Arc::new(GroupperAPI::new(
            tabster_core.clone(),
            tabster_core_ref.get_window.clone(),
        )));
    }

    tabster_core_ref.groupper.clone().unwrap_throw()
}
