use web_sys::{
    wasm_bindgen::{prelude::Closure, JsCast, UnwrapThrowExt},
    EventTarget, Node, Window,
};

pub fn set_timeout(window: &Window, handler: impl Fn() + 'static, timeout: i32) -> i32 {
    let handler = Box::new(handler) as Box<dyn Fn() + 'static>;
    let handler = Closure::once_into_js(handler);
    window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            handler.as_ref().unchecked_ref(),
            timeout,
        )
        .unwrap_throw()
}

pub fn add_event_listener<E>(
    target: impl Into<EventTarget>,
    event_name: &str,
    cb: impl Fn(E) + 'static,
) -> EventListenerHandle
where
    E: JsCast,
{
    add_event_listener_untyped_with_bool(
        target,
        event_name,
        move |e| cb(e.unchecked_into::<E>()),
        None,
    )
}

pub fn add_event_listener_with_bool<E>(
    target: impl Into<EventTarget>,
    event_name: &str,
    cb: impl Fn(E) + 'static,
    use_capture: bool,
) -> EventListenerHandle
where
    E: JsCast,
{
    add_event_listener_untyped_with_bool(
        target,
        event_name,
        move |e| cb(e.unchecked_into::<E>()),
        Some(use_capture),
    )
}

pub struct EventListenerHandle(Box<dyn FnOnce() + Send + Sync>);

impl std::fmt::Debug for EventListenerHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("EventListenerHandle").finish()
    }
}

impl EventListenerHandle {
    pub fn remove(self) {
        (self.0)();
    }
}

fn add_event_listener_untyped_with_bool(
    target: impl Into<EventTarget>,
    event_name: &str,
    cb: impl Fn(web_sys::Event) + 'static,
    use_capture: Option<bool>,
) -> EventListenerHandle {
    fn wel(
        target: EventTarget,
        cb: Box<dyn FnMut(web_sys::Event)>,
        event_name: &str,
        use_capture: Option<bool>,
    ) -> EventListenerHandle {
        let cb = Closure::wrap(cb).into_js_value();

        if let Some(use_capture) = use_capture {
            let _ = target.add_event_listener_with_callback_and_bool(
                event_name,
                cb.unchecked_ref(),
                use_capture,
            );
        } else {
            let _ = target.add_event_listener_with_callback(event_name, cb.unchecked_ref());
        }

        EventListenerHandle({
            let event_name = event_name.to_string();
            let cb = send_wrapper::SendWrapper::new(cb);
            let target = send_wrapper::SendWrapper::new(target);
            Box::new(move || {
                if let Some(use_capture) = use_capture {
                    let _ = target.remove_event_listener_with_callback_and_bool(
                        &event_name,
                        cb.unchecked_ref(),
                        use_capture,
                    );
                } else {
                    let _ =
                        target.remove_event_listener_with_callback(&event_name, cb.unchecked_ref());
                }
            })
        })
    }

    wel(target.into(), Box::new(cb), event_name, use_capture)
}

pub fn console_log_node(node: &Node) {
    web_sys::console::log_1(node);
}

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (web_sys::console::log_1(&web_sys::wasm_bindgen::JsValue::from_str(&format_args!($($t)*).to_string())))
}

#[macro_export]
macro_rules! console_error {
    ($($t:tt)*) => (web_sys::console::error_1(&web_sys::wasm_bindgen::JsValue::from_str(&format_args!($($t)*).to_string())))
}
