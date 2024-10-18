use crate::{tabster::TabsterCore, types, types::GetTabsterContextOptions};
use web_sys::{
    wasm_bindgen::{JsCast, UnwrapThrowExt},
    HtmlElement, Node, Window,
};

pub type WindowWithTabsterInstance = Window;

pub struct RootAPI;

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
        tabster: &mut TabsterCore,
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
        tabster.drain_init_queue();

        let root: Option<types::Root> = None;
        // let modalizer: Types.Modalizer | undefined;
        // let groupper: Types.Groupper | undefined;
        // let mover: Types.Mover | undefined;
        // let excludedFromMover = false;
        // let groupperBeforeMover: boolean | undefined;
        // let modalizerInGroupper: Types.Groupper | undefined;
        let mut dir_right_to_left: Option<bool> = None;
        // let uncontrolled: HTMLElement | null | undefined;
        let mut cur_element = Some(reference_element.map_or(element, |el| el.into()));
        // const ignoreKeydown: Types.FocusableProps["ignoreKeydown"] = {};

        loop {
            let Some(new_cur_element) = cur_element.clone() else {
                break;
            };
            if root.is_some() && check_rtl.unwrap_or_default() {
                break;
            }
            // const tabsterOnElement = getTabsterOnElement(
            //     tabster,
            //     curElement as HTMLElement
            // );

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

            // if (!tabsterOnElement) {
            //     curElement = getParent(curElement);
            //     continue;
            // }

            let tag_name = new_cur_element
                .clone()
                .dyn_into::<HtmlElement>()
                .unwrap_throw()
                .tag_name();

            // if (
            //     tabsterOnElement.uncontrolled ||
            //     tagName === "IFRAME" ||
            //     tagName === "WEBVIEW"
            // ) {
            //     uncontrolled = curElement as HTMLElement;
            // }

            // if (
            //     !mover &&
            //     tabsterOnElement.focusable?.excludeFromMover &&
            //     !groupper
            // ) {
            //     excludedFromMover = true;
            // }

            // const curModalizer = tabsterOnElement.modalizer;
            // const curGroupper = tabsterOnElement.groupper;
            // const curMover = tabsterOnElement.mover;

            // if (!modalizer && curModalizer) {
            //     modalizer = curModalizer;
            // }

            // if (!groupper && curGroupper && (!modalizer || curModalizer)) {
            //     if (modalizer) {
            //         // Modalizer dominates the groupper when they are on the same node and the groupper is active.
            //         if (
            //             !curGroupper.isActive() &&
            //             curGroupper.getProps().tabbability &&
            //             modalizer.userId !== tabster.modalizer?.activeId
            //         ) {
            //             modalizer = undefined;
            //             groupper = curGroupper;
            //         }

            //         modalizerInGroupper = curGroupper;
            //     } else {
            //         groupper = curGroupper;
            //     }
            // }

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

            cur_element = (tabster.get_parent)(new_cur_element);
        }

        // No root element could be found, try to get an auto root
        if root.is_none() {
            // const rootAPI = tabster.root as RootAPI;
            // const autoRoot = rootAPI._autoRoot;

            // if (autoRoot) {
            //     if (element.ownerDocument?.body) {
            //         root = rootAPI._autoRootCreate();
            //     }
            // }
        }

        // if (groupper && !mover) {
        //     groupperBeforeMover = true;
        // }

        // if (__DEV__ && !root) {
        //     if (modalizer || groupper || mover) {
        //         console.error(
        //             "Tabster Root is required for Mover, Groupper and Modalizer to work."
        //         );
        //     }
        // }

        // const shouldIgnoreKeydown = (event: KeyboardEvent) =>
        //     !!ignoreKeydown[
        //         event.key as keyof Types.FocusableProps["ignoreKeydown"]
        //     ];

        // return root
        //     ? {
        //           root,
        //           modalizer,
        //           groupper,
        //           mover,
        //           groupperBeforeMover,
        //           modalizerInGroupper,
        //           rtl: checkRtl ? !!dirRightToLeft : undefined,
        //           uncontrolled,
        //           excludedFromMover,
        //           ignoreKeydown: shouldIgnoreKeydown,
        //       }
        //     : undefined;

        None
    }
}
