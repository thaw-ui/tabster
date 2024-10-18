use std::{collections::HashMap, rc::Weak};

use crate::{
    dom_api::DOM,
    focusable::FocusableAPI,
    root::WindowWithTabsterInstance,
    types::{self, TabsterCoreProps, DOMAPI},
};
use web_sys::{wasm_bindgen::convert::ReturnWasmAbi, HtmlElement, Node, Window};

pub fn create_tabster(win: Window, props: TabsterCoreProps) -> Tabster {
    let tabster = TabsterCore::new(win, props);

    tabster.create_tabster()
}

pub struct Tabster {
    pub focusable: FocusableAPI,
}

impl Tabster {
    fn new(tabster: TabsterCore) -> Tabster {
        let focusable = FocusableAPI::new(tabster);

        Self { focusable }
    }
}

pub struct TabsterCore {
    storage: HashMap<HtmlElement, types::TabsterElementStorage>,
    win: Option<WindowWithTabsterInstance>,
    init_queue: Vec<Box<dyn FnOnce()>>,

    pub get_parent: Box<dyn Fn(Node) -> Option<Node>>,
}

impl TabsterCore {
    fn new(win: Window, props: TabsterCoreProps) -> Self {
        let get_parent = props
            .get_parent
            .unwrap_or_else(|| Box::new(move |node| DOM::get_parent_node(Some(node))));
        Self {
            storage: HashMap::new(),
            get_parent,
            win: Some(win),
            init_queue: Vec::new(),
        }
    }

    fn create_tabster(self) -> Tabster {
        Tabster::new(self)
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
        &self,
        element: HtmlElement,
        addremove: Option<bool>,
    ) -> Option<types::TabsterElementStorageEntry> {
        // let entry = self.storage.get(element.return_abi());

        // if (entry) {
        //     if (addremove === false && Object.keys(entry).length === 0) {
        //         storage.delete(element);
        //     }
        // } else if (addremove === true) {
        //     entry = {};
        //     storage.set(element, entry);
        // }

        // return entry;
        todo!()
    }
}
