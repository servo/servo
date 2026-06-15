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

use crate::bidi::{browsing_context, log, script};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum WebDriverMessage {
    FromConstellation(ConstellationToWebDriverMessage),
    FromScript(ScriptToWebDriverMessage),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ConstellationToWebDriverMessage {
    BrowsingContextCreated(browsing_context::Info),
    FromScript(),
}

// TODO: command responses need session id
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ScriptToWebDriverMessage {
    LogEntryAdded(Vec<BrowsingContextId>, log::EntryAdded),
    RealmCreated(script::RealmInfo),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum WebDriverToConstellationMessage {}

impl From<ConstellationToWebDriverMessage> for WebDriverMessage {
    fn from(value: ConstellationToWebDriverMessage) -> Self {
        Self::FromConstellation(value)
    }
}

impl From<ScriptToWebDriverMessage> for WebDriverMessage {
    fn from(value: ScriptToWebDriverMessage) -> Self {
        Self::FromScript(value)
    }
}
