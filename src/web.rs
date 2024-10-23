use web_sys::{
    wasm_bindgen::{prelude::Closure, JsCast, UnwrapThrowExt},
    Window,
};

pub fn set_timeout(window: &Window, handler: impl Fn() + 'static, timeout: i32) -> i32 {
    let handler = Box::new(handler) as Box<dyn Fn() + 'static>;
    let handler = Closure::wrap(handler);
    window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            handler.as_ref().unchecked_ref(),
            timeout,
        )
        .unwrap_throw()
}
