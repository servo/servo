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
}

use serde::{Deserialize, Serialize};
use servo_base::id::BrowsingContextId;

use crate::bidi::{browsing_context::Info, log::EntryAdded};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum WebDriverMessage {
    Constellation(ConstellationToWebDriverMessage),
    Script(ScriptToWebDriverMessage),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ConstellationToWebDriverMessage {
    BrowsingContextCreated(Info),
}

// TODO: command responses need session id
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ScriptToWebDriverMessage {
    EntryAdded(Vec<BrowsingContextId>, EntryAdded),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum WebDriverToConstellationMessage {}
