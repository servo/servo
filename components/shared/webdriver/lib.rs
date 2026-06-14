use serde::{Deserialize, Serialize};

pub mod bidi {
    use crate::bidi::script::{
        BigIntValue, BooleanValue, NullValue, NumberValue, NumberValueValue, ObjectRemoteValue,
        PrimitiveProtocolValue,
        SpecialNumber::{Infinity, NaN, NegInfinity, NegZero},
        StringValue, UndefinedValue,
    };

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
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ScriptToWebDriverMessage {
    ConsoleEntryAdded(bidi::log::ConsoleLogEntry),
}
