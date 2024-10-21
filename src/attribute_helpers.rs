use crate::{consts::TABSTER_ATTRIBUTE_NAME, types};

pub fn get_tabster_attribute(props: types::TabsterAttributeProps) -> types::TabsterDOMAttribute {
    (TABSTER_ATTRIBUTE_NAME.to_string(), props.json_string())
}
pub fn get_tabster_attribute_plain(props: types::TabsterAttributeProps) -> String {
    props.json_string()
}
