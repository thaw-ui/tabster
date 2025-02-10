use web_sys::{
    wasm_bindgen::{prelude::Closure, JsCast, UnwrapThrowExt},
    Node, Window,
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
