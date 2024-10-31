use crate::{
    console_log,
    dom_api::DOM,
    focusable::FocusableAPI,
    groupper::GroupperAPI,
    mover::MoverAPI,
    root::{RootAPI, WindowWithTabsterInstance},
    state::focused_element::FocusedElementState,
    types::{self, GetWindow, TabsterCoreProps, DOMAPI},
    web::set_timeout,
};
use std::{cell::RefCell, collections::HashMap, sync::Arc};
use web_sys::{js_sys::WeakMap, wasm_bindgen::UnwrapThrowExt, Node, Window};

thread_local! {
    static TABSTER_INSTANCE: RefCell<Option<Tabster>> = Default::default();
}

pub fn create_tabster(win: Window, props: TabsterCoreProps) -> Tabster {
    TABSTER_INSTANCE.with(|instance| {
        let mut instance = instance.borrow_mut();
        if let Some(instance) = instance.as_ref() {
            instance.clone()
        } else {
            let tabster = TabsterCore::new(win, props);
            let tabster = Tabster::new(tabster);
            *instance = Some(tabster.clone());
            tabster
        }
    })
}

#[derive(Clone)]
pub struct Tabster {
    pub focusable: Arc<RefCell<FocusableAPI>>,
    pub core: Arc<RefCell<TabsterCore>>,
}

impl Tabster {
    fn new(tabster: Arc<RefCell<TabsterCore>>) -> Tabster {
        let focusable = Arc::new(RefCell::new(FocusableAPI::new(tabster.clone())));
        {
            let mut tabster = tabster.borrow_mut();
            tabster.focusable = Some(focusable.clone());
        }

        Self {
            focusable,
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
    fn get_storage_value(&self, el: &Node) -> Option<String> {
        let value = self.storage.get(el);
        value.as_string()
    }
    fn get(&self, el: &Node) -> Option<Arc<RefCell<types::TabsterElementStorage>>> {
        let value = self.get_storage_value(el)?;
        self.data.get(&value).cloned()
    }

    fn set(&mut self, el: &Node, value: Arc<RefCell<types::TabsterElementStorage>>) {
        let uuid = uuid::Uuid::new_v4().to_string();
        self.storage
            .set(el, &web_sys::wasm_bindgen::JsValue::from_str(&uuid));
        self.data.insert(uuid, value);
    }

    fn delete(&mut self, el: &Node) {
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
    internal: Option<Arc<RefCell<types::InternalAPI>>>,
    pub focused_element: Option<FocusedElementState>,
    /// .unwrap
    pub focusable: Option<Arc<RefCell<FocusableAPI>>>,
    pub root: Option<RootAPI>,

    // Extended APIs
    pub groupper: Option<Arc<RefCell<GroupperAPI>>>,
    pub mover: Option<Arc<RefCell<MoverAPI>>>,
    pub modalizer: Option<types::ModalizerAPI>,
    pub get_parent: Box<dyn Fn(Node) -> Option<Node>>,
}

impl TabsterCore {
    fn new(win: Window, props: TabsterCoreProps) -> Arc<RefCell<Self>> {
        let get_parent = props
            .get_parent
            .unwrap_or_else(|| Box::new(move |node| DOM::get_parent_node(Some(node))));
        let get_window = {
            let win = win.clone();
            Arc::new(Box::new(move || win.clone()) as GetWindow)
        };
        let tabster = Arc::new(RefCell::new(Self {
            storage: TabsterCoreStorage::new(),
            get_parent,
            win: Some(win.clone()),
            noop: false,
            control_tab: props.control_tab.unwrap_or(true),
            get_window: get_window.clone(),
            internal: None,
            focused_element: None,
            focusable: None,
            root: None,
            groupper: None,
            mover: None,
            modalizer: None,
            init_queue: Default::default(),
            init_timer: Default::default(),
        }));

        let internal = Arc::new(RefCell::new(types::InternalAPI::new(win, tabster.clone())));
        let focused_element = FocusedElementState::new(tabster.clone(), get_window);
        let root = RootAPI::new(tabster.clone(), props.auto_root);
        {
            let mut tabster = tabster.borrow_mut();
            tabster.internal = Some(internal.clone());
            tabster.focused_element = Some(focused_element);
            tabster.root = Some(root);

            tabster.queue_init(move || {
                let mut internal = internal.try_borrow_mut().unwrap_throw();
                internal.resume_observer(true);
            });
        }

        tabster
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
        element: &Node,
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
pub fn get_groupper(tabster: &Tabster) -> Arc<RefCell<GroupperAPI>> {
    let tabster_core = tabster.core.clone();
    let mut tabster_core_ref = tabster_core.try_borrow_mut().unwrap_throw();

    if tabster_core_ref.groupper.is_none() {
        tabster_core_ref.groupper = Some(Arc::new(RefCell::new(GroupperAPI::new(
            tabster_core.clone(),
            tabster_core_ref.get_window.clone(),
        ))));
    }

    tabster_core_ref.groupper.clone().unwrap_throw()
}

/// Creates a new mover instance or returns an existing one
/// @param tabster Tabster instance
pub fn get_mover(tabster: &Tabster) -> Arc<RefCell<MoverAPI>> {
    console_log!("get_mover");
    let tabster_core = tabster.core.clone();
    let mut tabster_core_ref = tabster_core.try_borrow_mut().unwrap_throw();

    if tabster_core_ref.mover.is_none() {
        console_log!("get_mover is_none");
        tabster_core_ref.mover = Some(Arc::new(RefCell::new(MoverAPI::new(
            tabster_core.clone(),
            tabster_core_ref.get_window.clone(),
        ))));
    }

    tabster_core_ref.mover.clone().unwrap_throw()
}
