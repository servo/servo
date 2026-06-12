use serde::{Deserialize, Serialize};

pub mod bidi {
    include!(concat!(env!("OUT_DIR"), "/webdriver_bidi.rs"));

    impl Default for script::SerializationOptions {
        fn default() -> Self {
            Self {
                max_dom_depth: Some(0),
                max_object_depth: None,
                include_shadow_tree: Some(script::SerializationOptionsIncludeShadowTree::None),
            }
        }
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#serialize-as-a-remote-value>
    pub fn serialize_as_a_remote_value(
        value: impl SerializeRemoteValue,
        serialization_options: &script::SerializationOptions,
        ownership_type: script::ResultOwnership,
        // serialization_internal_map: (),
        // realm: String,
        // session: String,
    ) -> script::RemoteValue {
        todo!()
    }

    /// A value that can be serialized by [`serialize_as_a_remote_value`]
    pub trait SerializeRemoteValue {
        fn is_undefined(&self) -> bool;
        fn is_symbol(&self) -> bool;
        fn is_array(&self) -> bool;
        fn is_regexp(&self) -> bool;
        fn is_date(&self) -> bool;
        fn is_map(&self) -> bool;
        fn is_set(&self) -> bool;
        fn is_weak_map(&self) -> bool;
        fn is_weak_set(&self) -> bool;
        fn is_generator(&self) -> bool;
        fn is_error(&self) -> bool;
        fn is_proxy(&self) -> bool;
        fn is_promise(&self) -> bool;
        fn is_type_array(&self) -> bool;
        fn is_array_buffer(&self) -> bool;
        fn is_node_list(&self) -> bool;
        fn is_html_collection(&self) -> bool;
        fn is_node(&self) -> bool;
        fn is_window_proxy(&self) -> bool;
        fn is_platform_object(&self) -> bool;
        fn is_callable(&self) -> bool;
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#get-the-source>
    pub fn get_the_source() -> script::Source {
        todo!()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ScriptToWebDriverMessage {
    ConsoleEntryAdded(bidi::log::ConsoleLogEntry),
}
