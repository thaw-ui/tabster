use web_sys::{wasm_bindgen::UnwrapThrowExt, HtmlElement};

use crate::{console_error, consts::TABSTER_ATTRIBUTE_NAME, types};

pub fn get_tabster_attribute(props: types::TabsterAttributeProps) -> types::TabsterDOMAttribute {
    (TABSTER_ATTRIBUTE_NAME.to_string(), props.json_string())
}
pub fn get_tabster_attribute_plain(props: &types::TabsterAttributeProps) -> String {
    serde_json::to_string(&props).unwrap_throw()
}

/// Updates Tabster props object with new props.
/// @param element an element to set data-tabster attribute on.
/// @param props current Tabster props to update.
/// @param newProps new Tabster props to add.
///  When the value of a property in newProps is undefined, the property
///  will be removed from the attribute.
pub fn merge_tabster_props(
    props: &mut types::TabsterAttributeProps,
    new_props: types::TabsterAttributeProps,
) {
    *props = new_props;
}

/// Sets or updates Tabster attribute of the element.
/// @param element an element to set data-tabster attribute on.
/// @param newProps new Tabster props to set.
/// @param update if true, newProps will be merged with the existing props.
///  When true and the value of a property in newProps is undefined, the property
///  will be removed from the attribute.
pub fn set_tabster_attribute(
    element: HtmlElement,
    new_props: types::TabsterAttributeProps,
    update: Option<bool>,
) {
    let mut props: Option<types::TabsterAttributeProps> = None;

    if update.unwrap_or_default() {
        if let Some(attr) = element.get_attribute(TABSTER_ATTRIBUTE_NAME) {
            match serde_json::from_str::<types::TabsterAttributeProps>(&attr) {
                Ok(new_value) => {
                    props = Some(new_value);
                }
                Err(err) => {
                    console_error!("data-tabster attribute error: {}", err.to_string());
                }
            }
        }
    }

    let mut props = if let Some(props) = props {
        props
    } else {
        types::TabsterAttributeProps::default()
    };

    merge_tabster_props(&mut props, new_props);

    if !props.is_empty() {
        element
            .set_attribute(TABSTER_ATTRIBUTE_NAME, &get_tabster_attribute_plain(&props))
            .unwrap_throw();
    } else {
        element
            .remove_attribute(TABSTER_ATTRIBUTE_NAME)
            .unwrap_throw();
    }
}
