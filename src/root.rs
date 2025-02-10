use crate::{
    groupper::Groupper,
    instance::{get_tabster_on_element, update_tabster_by_attribute},
    modalizer::{ArcCellModalizer, Modalizer},
    mover::Mover,
    set_tabster_attribute,
    tabster::TabsterCore,
    types::{self, GetTabsterContextOptions, Root, TabsterContext},
};
use std::{cell::RefCell, sync::Arc};
use web_sys::{
    wasm_bindgen::{JsCast, UnwrapThrowExt},
    Document, HtmlElement, KeyboardEvent, Node, Window,
};

pub type WindowWithTabsterInstance = Window;

pub struct RootAPI {
    tabster: Arc<RefCell<TabsterCore>>,
    win: Arc<Box<dyn Fn() -> Window>>,
    auto_root_waiting: bool,
    auto_root: Option<types::RootProps>,
}

impl RootAPI {
    pub fn new(tabster: Arc<RefCell<TabsterCore>>, auto_root: Option<types::RootProps>) -> Self {
        let win = {
            let tabster = tabster.borrow();
            tabster.get_window.clone()
        };
        Self {
            tabster,
            win,
            auto_root,
            auto_root_waiting: false,
        }
    }

    fn auto_root_create(&mut self) -> Option<Arc<types::Root>> {
        let doc = (self.win)().document().unwrap_throw();
        let body = doc.body();

        if let Some(body) = body {
            self.auto_root_unwait(doc);

            if let Some(props) = &self.auto_root {
                let mut new_props = types::TabsterAttributeProps::default();
                new_props.root = Some(props.clone());
                set_tabster_attribute(body.clone(), new_props, Some(true));
                update_tabster_by_attribute(&self.tabster, &body, None);
                let Some(tabster_on_element) = get_tabster_on_element(&self.tabster, &body) else {
                    return None;
                };
                let tabster_on_element = tabster_on_element.borrow();
                return tabster_on_element.root.clone();
            }
        } else if !self.auto_root_waiting {
            self.auto_root_waiting = true;
            // doc.addEventListener("readystatechange", this._autoRootCreate);
        }

        None
    }

    fn auto_root_unwait(&mut self, doc: Document) {
        // doc.removeEventListener("readystatechange", this._autoRootCreate);
        self.auto_root_waiting = false;
    }
}

impl RootAPI {
    /// Fetches the tabster context for an element walking up its ancestors
    ///
    /// tabster: Tabster instance
    ///
    /// element: The element the tabster context should represent
    ///
    /// options: Additional options
    ///
    /// returns: None if the element is not a child of a tabster root, otherwise all applicable tabster behaviours and configurations
    pub fn get_tabster_context(
        tabster: &Arc<RefCell<TabsterCore>>,
        element: &Node,
        options: GetTabsterContextOptions,
    ) -> Option<types::TabsterContext> {
        if element.owner_document().is_none() {
            return None;
        };

        let GetTabsterContextOptions {
            check_rtl,
            reference_element,
        } = options;

        // Normally, the initialization starts on the next tick after the tabster
        // instance creation. However, if the application starts using it before
        // the next tick, we need to make sure the initialization is done.
        {
            let mut tabster = tabster.try_borrow_mut().unwrap_throw();
            tabster.drain_init_queue();
        }

        let mut root: Option<Arc<types::Root>> = None;
        let mut modalizer = None::<ArcCellModalizer>;
        let mut groupper = None::<Arc<RefCell<Groupper>>>;
        let mut mover = None::<Arc<RefCell<Mover>>>;
        let mut excluded_from_mover = false;
        let mut groupper_before_mover = None::<bool>;
        let mut modalizer_in_groupper = None::<Arc<RefCell<Groupper>>>;
        let mut dir_right_to_left: Option<bool> = None;
        let mut uncontrolled = None::<HtmlElement>;
        let mut cur_element = Some(reference_element.map_or(element.clone(), |el| el.into()));
        let mut ignore_keydown = types::IgnoreKeydown::default(); // Types.FocusableProps["ignoreKeydown"] = {};

        loop {
            let Some(new_cur_element) = cur_element.clone() else {
                break;
            };
            if root.is_some() && check_rtl.unwrap_or_default() {
                break;
            }
            let tabster_on_element = get_tabster_on_element(&tabster, &new_cur_element.clone());

            if check_rtl.unwrap_or_default() && dir_right_to_left.is_none() {
                let dir = new_cur_element
                    .clone()
                    .dyn_into::<HtmlElement>()
                    .unwrap_throw()
                    .dir();

                if !dir.is_empty() {
                    dir_right_to_left = Some(dir.to_lowercase() == "rtl");
                }
            }

            let Some(tabster_on_element) = tabster_on_element else {
                {
                    let tabster = tabster.borrow();
                    cur_element = (tabster.get_parent)(new_cur_element);
                }
                continue;
            };

            let tag_name = new_cur_element
                .clone()
                .dyn_into::<HtmlElement>()
                .unwrap_throw()
                .tag_name();

            let tabster_on_element = tabster_on_element.borrow();
            if tabster_on_element.uncontrolled.is_some()
                || tag_name == "IFRAME"
                || tag_name == "WEBVIEW"
            {
                uncontrolled = Some(new_cur_element.clone().dyn_into().unwrap_throw());
            }

            if mover.is_none() && groupper.is_none() {
                if let Some(focusable) = tabster_on_element.focusable.as_ref() {
                    if focusable.exclude_from_mover.unwrap_or_default() {
                        excluded_from_mover = true;
                    }
                }
            }

            let cur_modalizer = &tabster_on_element.modalizer;
            let cur_groupper = &tabster_on_element.groupper;
            let cur_mover = &tabster_on_element.mover;

            if modalizer.is_none() {
                if let Some(cur_modalizer) = cur_modalizer {
                    modalizer = Some(cur_modalizer.clone());
                }
            }

            if groupper.is_none() && (modalizer.is_none() || cur_modalizer.is_some()) {
                if let Some(cur_groupper) = cur_groupper {
                    if modalizer.is_some() {
                        let mut cur_groupper_ref = cur_groupper.borrow_mut();

                        let user_id = {
                            if let Some(modalizer) = modalizer.as_ref() {
                                Some(modalizer.borrow().user_id.clone())
                            } else {
                                None
                            }
                        };
                        let active_id = {
                            let tabster = tabster.borrow();
                            if let Some(modalizer) = tabster.modalizer.as_ref() {
                                modalizer.active_id.clone()
                            } else {
                                None
                            }
                        };
                        // Modalizer dominates the groupper when they are on the same node and the groupper is active.
                        if !cur_groupper_ref.is_active(None).unwrap_or_default()
                            && cur_groupper_ref.get_props().tabbability.unwrap_or_default() != 0
                            && user_id != active_id
                        {
                            modalizer = None;
                            groupper = Some(cur_groupper.clone());
                        }
                        modalizer_in_groupper = Some(cur_groupper.clone());
                    } else {
                        groupper = Some(cur_groupper.clone());
                    }
                }
            }
            if mover.is_none()
                && cur_mover.is_some()
                && (modalizer.is_none() || cur_modalizer.is_some())
                && (cur_groupper.is_none() || cur_element != Some(element.clone()))
                && cur_element
                    .clone()
                    .map(|el| el.contains(Some(&element)))
                    .unwrap_or_default()
            // Mover makes sense only for really inside elements, not for virutal out of the DOM order children.
            {
                mover = cur_mover.clone();
                groupper_before_mover = if let Some(groupper) = groupper.as_ref() {
                    if let Some(cur_groupper) = cur_groupper {
                        Some(!Arc::ptr_eq(&groupper, cur_groupper))
                    } else {
                        Some(false)
                    }
                } else {
                    Some(false)
                };
            }

            if let Some(tabster_on_element_root) = tabster_on_element.root.clone() {
                root = Some(tabster_on_element_root);
            }

            if let Some(tabster_on_element_focusable) = tabster_on_element.focusable.clone() {
                if let Some(focusable_ignore_keydown) =
                    tabster_on_element_focusable.ignore_keydown.clone()
                {
                    ignore_keydown.assign(focusable_ignore_keydown);
                }
            }

            cur_element = {
                let tabster = tabster.borrow();
                (tabster.get_parent)(new_cur_element)
            }
        }

        // No root element could be found, try to get an auto root
        if root.is_none() {
            let mut tabster = tabster.borrow_mut();
            if let Some(root_api) = tabster.root.as_mut() {
                if root_api.auto_root.is_some() {
                    if let Some(owner_document) = element.owner_document() {
                        if owner_document.body().is_some() {
                            root = root_api.auto_root_create();
                        }
                    }
                }
            }
        }

        if groupper.is_some() && mover.is_none() {
            groupper_before_mover = Some(true);
        }

        #[cfg(debug_assertions)]
        if root.is_none() {
            if modalizer.is_some() || groupper.is_some() || mover.is_some() {
                web_sys::console::error_1(&web_sys::wasm_bindgen::JsValue::from_str(
                    "Tabster Root is required for Mover, Groupper and Modalizer to work.",
                ));
            }
        }

        let should_ignore_keydown = move |event: KeyboardEvent| {
            let key = event.key();
            ignore_keydown.get(&key).unwrap_or_default()
        };

        if let Some(root) = root {
            Some(TabsterContext {
                root,
                groupper_before_mover,
                rtl: check_rtl
                    .unwrap_or_default()
                    .then(|| dir_right_to_left.unwrap_or_default()),
                excluded_from_mover: Some(excluded_from_mover),
                uncontrolled,
                ignore_keydown: Arc::new(should_ignore_keydown),
                modalizer,
                groupper,
                mover,
                modalizer_in_groupper,
            })
        } else {
            None
        }
    }

    pub(crate) fn get_root(
        tabster: &Arc<RefCell<TabsterCore>>,
        element: HtmlElement,
    ) -> Option<Arc<Root>> {
        let mut el = Some(element);
        while let Some(new_el) = el.clone() {
            let root = get_tabster_on_element(tabster, &new_el)
                .map(|tabster_on_element| tabster_on_element.borrow().root.clone())
                .flatten();

            if root.is_some() {
                return root;
            }

            let tabster = tabster.borrow();
            el = (tabster.get_parent)(new_el.into()).map(|el| el.dyn_into().unwrap_throw());
        }

        None
    }
}
