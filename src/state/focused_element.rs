use web_sys::HtmlElement;

use crate::{
    dom_api::DOM,
    groupper::ArcCellGroupper,
    modalizer::ArcCellModalizer,
    mover::ArcCellMover,
    root::RootAPI,
    tabster::TabsterCore,
    types::{self, GetTabsterContextOptions, NextTabbable, DOMAPI},
    utils::get_last_child,
    web::set_timeout,
};
use std::{
    cell::RefCell,
    sync::{
        atomic::{AtomicBool, AtomicI32, Ordering},
        Arc,
    },
};

static FOCUSED_ELEMENT_STATE_IS_TABBING: AtomicBool = AtomicBool::new(false);
static FOCUSED_ELEMENT_STATE_IS_TABBING_TIMER: AtomicI32 = AtomicI32::new(0);

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
        let actual_container = if let Some(container) = container.clone() {
            container
        } else if let Some(el) = ctx.root.get_element() {
            el
        } else {
            return None;
        };

        let mut next = None::<NextTabbable>;
        let is_tabbing_timer = FOCUSED_ELEMENT_STATE_IS_TABBING_TIMER.load(Ordering::SeqCst);
        let win = (tabster.borrow().get_window)();

        if is_tabbing_timer != 0 {
            win.clear_timeout_with_handle(is_tabbing_timer);
        }

        FOCUSED_ELEMENT_STATE_IS_TABBING.store(true, Ordering::SeqCst);
        FOCUSED_ELEMENT_STATE_IS_TABBING_TIMER.store(
            set_timeout(
                &win,
                || {
                    FOCUSED_ELEMENT_STATE_IS_TABBING_TIMER.store(0, Ordering::SeqCst);
                    FOCUSED_ELEMENT_STATE_IS_TABBING.store(false, Ordering::SeqCst);
                },
                0,
            ),
            Ordering::SeqCst,
        );

        // const modalizer = ctx.modalizer;
        // const groupper = ctx.groupper;
        // const mover = ctx.mover;

        enum What {
            Groupper(ArcCellGroupper),
            Mover(ArcCellMover),
            Modalizer(ArcCellModalizer),
        }

        impl What {
            fn find_next_tabbable(
                &mut self,
                current_element: Option<HtmlElement>,
                reference_element: Option<HtmlElement>,
                is_backward: Option<bool>,
                ignore_accessibility: Option<bool>,
            ) -> Option<types::NextTabbable> {
                match self {
                    What::Groupper(groupper) => groupper.borrow_mut().find_next_tabbable(
                        current_element,
                        reference_element,
                        is_backward,
                        ignore_accessibility,
                    ),
                    What::Mover(mover) => mover.borrow_mut().find_next_tabbable(
                        current_element,
                        reference_element,
                        is_backward,
                        ignore_accessibility,
                    ),
                    What::Modalizer(modalizer) => modalizer.borrow_mut().find_next_tabbable(
                        current_element,
                        reference_element,
                        is_backward,
                        ignore_accessibility,
                    ),
                }
            }

            fn get_element(&self) -> Option<HtmlElement> {
                match self {
                    What::Groupper(groupper) => groupper.borrow().get_element(),
                    What::Mover(mover) => mover.borrow().get_element(),
                    What::Modalizer(modalizer) => modalizer.borrow().get_element(),
                }
            }
        }

        let call_find_next = {
            let current_element = current_element.clone();
            let reference_element = reference_element.clone();
            move |next: &mut Option<NextTabbable>, mut what: What| {
                *next = what.find_next_tabbable(
                    current_element.clone(),
                    reference_element,
                    is_backward,
                    ignore_accessibility,
                );

                let Some(current_element) = current_element else {
                    return;
                };

                if let Some(next) = next.as_ref() {
                    if next.element.is_some() {
                        return;
                    }
                } else {
                    return;
                }

                let parent_element = match &what {
                    What::Modalizer(_) => None,
                    What::Groupper(groupper) => {
                        DOM::get_parent_element(groupper.borrow().get_element())
                    }
                    What::Mover(mover) => DOM::get_parent_element(mover.borrow().get_element()),
                };

                let Some(parent_element) = parent_element else {
                    return;
                };

                let parent_ctx = RootAPI::get_tabster_context(
                    tabster,
                    &current_element,
                    GetTabsterContextOptions {
                        reference_element: Some(parent_element.clone()),
                        check_rtl: None,
                    },
                );

                let Some(parent_ctx) = parent_ctx else {
                    return;
                };

                let current_scope_element = what.get_element();
                let new_current = if is_backward.unwrap_or_default() {
                    current_scope_element
                } else {
                    if let Some(current_scope_element) = current_scope_element {
                        Some(
                            get_last_child(&current_scope_element).unwrap_or(current_scope_element),
                        )
                    } else {
                        current_scope_element
                    }
                };

                if let Some(new_current) = new_current {
                    *next = FocusedElementState::find_next_tabbable(
                        tabster,
                        parent_ctx,
                        container,
                        Some(new_current),
                        Some(parent_element),
                        is_backward,
                        ignore_accessibility,
                    );

                    if let Some(next) = next {
                        next.out_of_dom_order = Some(true);
                    }
                }
            }
        };

        if ctx.groupper.is_some() && ctx.mover.is_some() {
            call_find_next(
                &mut next,
                if ctx.groupper_before_mover.unwrap_or_default() {
                    What::Groupper(ctx.groupper.clone().unwrap())
                } else {
                    What::Mover(ctx.mover.clone().unwrap())
                },
            );
        } else if let Some(groupper) = ctx.groupper {
            call_find_next(&mut next, What::Groupper(groupper));
        } else if let Some(mover) = ctx.mover {
            call_find_next(&mut next, What::Mover(mover));
        } else if let Some(modalizer) = ctx.modalizer {
            call_find_next(&mut next, What::Modalizer(modalizer));
        } else {
            let find_props = types::FindNextProps {
                container: actual_container,
                current_element,
                reference_element,
                ignore_accessibility,
                use_active_modalizer: Some(true),
            };

            let mut find_props_out = types::FindFocusableOutputProps::default();
            let tabster = tabster.borrow();
            let mut focusable = tabster.focusable.as_ref().unwrap().borrow_mut();
            let next_element = if is_backward.unwrap_or_default() {
                focusable.find_prev(find_props, &mut find_props_out)
            } else {
                focusable.find_next(find_props, &mut find_props_out)
            };

            next = Some(NextTabbable {
                element: next_element,
                out_of_dom_order: find_props_out.out_of_dom_order,
                uncontrolled: find_props_out.uncontrolled,
            });
        }

        next
    }
}
