use web_sys::{wasm_bindgen::UnwrapThrowExt, HtmlElement};

pub fn native_focus(element: HtmlElement) {
    element.focus().unwrap_throw();
}
