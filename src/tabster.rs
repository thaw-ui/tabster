use std::{cell::RefCell, collections::HashMap, sync::Arc};

use crate::{
    dom_api::DOM,
    focusable::FocusableAPI,
    root::{RootAPI, WindowWithTabsterInstance},
    types::{self, TabsterCoreProps, DOMAPI},
};
use web_sys::{js_sys::WeakMap, HtmlElement, Node, Window};

pub fn create_tabster(win: Window, props: TabsterCoreProps) -> Tabster {
    let tabster = TabsterCore::new(win, props);
    Tabster::new(tabster)
}

pub struct Tabster {
    pub focusable: FocusableAPI,
    pub root: RootAPI,
}

impl Tabster {
    fn new(tabster: TabsterCore) -> Tabster {
        let tabster = Arc::new(RefCell::new(tabster));
        let focusable = FocusableAPI::new(tabster.clone());
        let root = RootAPI::new(tabster);

        Self { focusable, root }
    }
}

// TODO Memory leak
struct TabsterCoreStorage {
    storage: web_sys::js_sys::WeakMap,
    data: HashMap<String, Arc<types::TabsterElementStorage>>,
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
    fn get(&self, el: &HtmlElement) -> Option<Arc<types::TabsterElementStorage>> {
        let value = self.get_storage_value(el)?;
        self.data.get(&value).cloned()
    }

    fn set(&mut self, el: &HtmlElement, value: Arc<types::TabsterElementStorage>) {
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
    init_queue: Vec<Box<dyn FnOnce()>>,

    // Extended APIs
    pub modalizer: Option<types::ModalizerAPI>,
    pub get_parent: Box<dyn Fn(Node) -> Option<Node>>,
}

impl TabsterCore {
    fn new(win: Window, props: TabsterCoreProps) -> Self {
        let get_parent = props
            .get_parent
            .unwrap_or_else(|| Box::new(move |node| DOM::get_parent_node(Some(node))));
        Self {
            storage: TabsterCoreStorage::new(),
            get_parent,
            win: Some(win),
            modalizer: None,
            init_queue: Vec::new(),
        }
    }

    pub fn drain_init_queue(&mut self) {
        if self.win.is_none() {
            return;
        }

        // Resetting the queue before calling the callbacks to avoid recursion.
        let queue = self.init_queue.drain(..);
        queue.into_iter().for_each(|callback| callback());
    }

    pub fn storage_entry(
        &mut self,
        element: &HtmlElement,
        addremove: Option<bool>,
    ) -> Option<Arc<types::TabsterElementStorageEntry>> {
        let mut entry = self.storage.get(element);
        if let Some(entry) = entry.as_ref() {
            if matches!(addremove, Some(false)) && entry.is_empty() {
                self.storage.delete(element);
            }
        } else if matches!(addremove, Some(true)) {
            entry = Some(Arc::new(types::TabsterElementStorageEntry::new()));
            self.storage.set(element, entry.clone().unwrap());
        }

        entry
    }
}
