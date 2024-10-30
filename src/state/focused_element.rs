use web_sys::HtmlElement;

use crate::{
    tabster::TabsterCore,
    types::{self, NextTabbable},
};
use std::{cell::RefCell, sync::Arc};

pub struct FocusedElementState {
    tabster: Arc<RefCell<TabsterCore>>,
    win: Arc<types::GetWindow>,
}

impl FocusedElementState {
    pub fn new(tabster: Arc<RefCell<TabsterCore>>, get_window: Arc<types::GetWindow>) -> Self {
        Self {
            tabster,
            win: get_window,
        }
    }

    pub fn get_focused_element(&self) -> Option<HtmlElement> {
        // TODO
        None
    }

    pub fn find_next_tabbable(
        tabster: &Arc<RefCell<TabsterCore>>,
        ctx: types::TabsterContext,
        container: Option<HtmlElement>,
        current_element: Option<HtmlElement>,
        reference_element: Option<HtmlElement>,
        is_backward: Option<bool>,
        ignore_accessibility: Option<bool>,
    ) -> Option<types::NextTabbable> {
        let actualContainer = if let Some(container) = container {
            container
        } else if let Some(el) = ctx.root.get_element() {
            el
        } else {
            return None;
        };

        let next = None::<NextTabbable>;
        // const isTabbingTimer = FocusedElementState._isTabbingTimer;
        // const win = tabster.getWindow();

        // if (isTabbingTimer) {
        //     win.clearTimeout(isTabbingTimer);
        // }

        // FocusedElementState.isTabbing = true;
        // FocusedElementState._isTabbingTimer = win.setTimeout(() => {
        //     delete FocusedElementState._isTabbingTimer;
        //     FocusedElementState.isTabbing = false;
        // }, 0);

        // const modalizer = ctx.modalizer;
        // const groupper = ctx.groupper;
        // const mover = ctx.mover;

        // const callFindNext = (
        //     what: Types.Groupper | Types.Mover | Types.Modalizer
        // ) => {
        //     next = what.findNextTabbable(
        //         currentElement,
        //         referenceElement,
        //         isBackward,
        //         ignoreAccessibility
        //     );

        //     if (currentElement && !next?.element) {
        //         const parentElement =
        //             what !== modalizer &&
        //             dom.getParentElement(what.getElement());

        //         if (parentElement) {
        //             const parentCtx = RootAPI.getTabsterContext(
        //                 tabster,
        //                 currentElement,
        //                 { referenceElement: parentElement }
        //             );

        //             if (parentCtx) {
        //                 const currentScopeElement = what.getElement();
        //                 const newCurrent = isBackward
        //                     ? currentScopeElement
        //                     : (currentScopeElement &&
        //                           getLastChild(currentScopeElement)) ||
        //                       currentScopeElement;

        //                 if (newCurrent) {
        //                     next = FocusedElementState.findNextTabbable(
        //                         tabster,
        //                         parentCtx,
        //                         container,
        //                         newCurrent,
        //                         parentElement,
        //                         isBackward,
        //                         ignoreAccessibility
        //                     );

        //                     if (next) {
        //                         next.outOfDOMOrder = true;
        //                     }
        //                 }
        //             }
        //         }
        //     }
        // };

        // if (groupper && mover) {
        //     callFindNext(ctx.groupperBeforeMover ? groupper : mover);
        // } else if (groupper) {
        //     callFindNext(groupper);
        // } else if (mover) {
        //     callFindNext(mover);
        // } else if (modalizer) {
        //     callFindNext(modalizer);
        // } else {
        //     const findProps: Types.FindNextProps = {
        //         container: actualContainer,
        //         currentElement,
        //         referenceElement,
        //         ignoreAccessibility,
        //         useActiveModalizer: true,
        //     };

        //     const findPropsOut: Types.FindFocusableOutputProps = {};

        //     const nextElement = tabster.focusable[
        //         isBackward ? "findPrev" : "findNext"
        //     ](findProps, findPropsOut);

        //     next = {
        //         element: nextElement,
        //         outOfDOMOrder: findPropsOut.outOfDOMOrder,
        //         uncontrolled: findPropsOut.uncontrolled,
        //     };
        // }

        // return next;

        None
    }
}
