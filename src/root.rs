use crate::{
    groupper::Groupper,
    instance::get_tabster_on_element,
    modalizer::Modalizer,
    mover::Mover,
    tabster::TabsterCore,
    types::{self, GetTabsterContextOptions, TabsterContext},
};
use std::{cell::RefCell, sync::Arc};
use web_sys::{
    wasm_bindgen::{JsCast, UnwrapThrowExt},
    HtmlElement, KeyboardEvent, Node, Window,
};

pub type WindowWithTabsterInstance = Window;

pub struct RootAPI {
    tabster: Arc<RefCell<TabsterCore>>,
}

impl RootAPI {
    pub fn new(tabster: Arc<RefCell<TabsterCore>>) -> Self {
        Self { tabster }
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
        tabster: Arc<RefCell<TabsterCore>>,
        element: Node,
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

        let root: Option<types::Root> = None;
        let mut modalizer = None::<Modalizer>;
        let groupper = None::<Groupper>;
        let mover = None::<Mover>;
        let mut excluded_from_mover = false;
        let mut groupper_before_mover = None::<bool>;
        let modalizer_in_groupper = None::<Groupper>;
        let mut dir_right_to_left: Option<bool> = None;
        let mut uncontrolled = None::<HtmlElement>;
        let mut cur_element = Some(reference_element.map_or(element, |el| el.into()));
        let ignore_keydown = types::IgnoreKeydown::default(); // Types.FocusableProps["ignoreKeydown"] = {};

        loop {
            let Some(new_cur_element) = cur_element.clone() else {
                break;
            };
            if root.is_some() && check_rtl.unwrap_or_default() {
                break;
            }
            let tabster_on_element = get_tabster_on_element(
                tabster.clone(),
                &new_cur_element.clone().dyn_into().unwrap_throw(),
            );

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
                    // modalizer = Some(cur_modalizer);
                }
            }

            if groupper.is_none()
                && cur_groupper.is_some()
                && (modalizer.is_none() || cur_modalizer.is_some())
            {
                if (modalizer.is_some()) {
                    // Modalizer dominates the groupper when they are on the same node and the groupper is active.
                    //         if (
                    //             !curGroupper.isActive() &&
                    //             curGroupper.getProps().tabbability &&
                    //             modalizer.userId !== tabster.modalizer?.activeId
                    //         ) {
                    //             modalizer = undefined;
                    //             groupper = curGroupper;
                    //         }

                    //         modalizerInGroupper = curGroupper;
                } else {
                    //         groupper = curGroupper;
                }
            }

            // if (
            //     !mover &&
            //     curMover &&
            //     (!modalizer || curModalizer) &&
            //     (!curGroupper || curElement !== element) &&
            //     curElement.contains(element) // Mover makes sense only for really inside elements, not for virutal out of the DOM order children.
            // ) {
            //     mover = curMover;
            //     groupperBeforeMover = !!groupper && groupper !== curGroupper;
            // }

            // if (tabsterOnElement.root) {
            //     root = tabsterOnElement.root;
            // }

            // if (tabsterOnElement.focusable?.ignoreKeydown) {
            //     Object.assign(
            //         ignoreKeydown,
            //         tabsterOnElement.focusable.ignoreKeydown
            //     );
            // }

            cur_element = {
                let tabster = tabster.borrow();
                (tabster.get_parent)(new_cur_element)
            }
        }

        // No root element could be found, try to get an auto root
        if root.is_none() {
            let tabster = tabster.borrow();
            // let rootAPI = tabster.root;
            // const autoRoot = rootAPI._autoRoot;

            // if (autoRoot) {
            //     if (element.ownerDocument?.body) {
            //         root = rootAPI._autoRootCreate();
            //     }
            // }
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
                ignore_keydown: Box::new(should_ignore_keydown),
                //           modalizer,
                //           groupper,
                //           mover,
                //           modalizerInGroupper,
            })
        } else {
            None
        }
    }
}
