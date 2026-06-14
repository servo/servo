use serde::{Deserialize, Serialize};
use servo_base::id::BrowsingContextId;

use crate::bidi::log::EntryAdded;

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

// TODO: this is intended for both classic and bidi,
// however, classic is not refactored yet.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ScriptToWebDriverMessage {
    EntryAdded(Vec<BrowsingContextId>, EntryAdded),
}
