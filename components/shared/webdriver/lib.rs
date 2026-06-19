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

    impl Default for storage::CookieFilter {
        fn default() -> Self {
            Self {
                name: Default::default(),
                value: Default::default(),
                domain: Default::default(),
                path: Default::default(),
                size: Default::default(),
                http_only: Default::default(),
                secure: Default::default(),
                same_site: Default::default(),
                expiry: Default::default(),
                extensible: Default::default(),
            }
        }
    }

    impl Default for EmptyResult {
        fn default() -> Self {
            Self {
                extensible: Default::default(),
            }
        }
    }

    impl CommandData {
        pub fn is_static(&self) -> bool {
            if let CommandData::SessionCommand(cmd) = self
                && let SessionCommand::New(_) | SessionCommand::Status(_) = cmd
            {
                true
            } else {
                false
            }
        }
    }
}

use devtools_traits::WorkerId;
use serde::{Deserialize, Serialize};
use servo_base::{
    generic_channel::{GenericCallback, GenericOneshotSender, GenericSender},
    id::{BrowsingContextId, PipelineId, WebViewId},
};
use uuid::Uuid;

use crate::bidi::{
    browsing_context::{self, ReadinessState},
    input, log, script,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum WebDriverMsg {
    FromConstellation(ConstellationToWebDriverMsg),
    FromScript(ScriptToWebDriverMsg),
}

impl From<ConstellationToWebDriverMsg> for WebDriverMsg {
    fn from(value: ConstellationToWebDriverMsg) -> Self {
        Self::FromConstellation(value)
    }
}

impl From<ScriptToWebDriverMsg> for WebDriverMsg {
    fn from(value: ScriptToWebDriverMsg) -> Self {
        Self::FromScript(value)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ConstellationToWebDriverMsg {
    BrowsingContextCreated(browsing_context::Info),
}

// TODO: command responses need session id
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ScriptToWebDriverMsg {
    LogEntryAdded(Vec<BrowsingContextId>, log::EntryAdded),
    RealmCreated(
        (BrowsingContextId, PipelineId, Option<WorkerId>, WebViewId),
        GenericSender<WebDriverToScriptMessage>,
    ),
    ScriptMessage {
        // TODO: channel should have more speicific id type
        channel: String,
        data: script::RemoteValue, // TODO: source with realm id & context id
    },
    FileDialogOpened(input::FileDialogOpened),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum WebDriverToConstellationMsg {
    Activate(BrowsingContextId, GenericCallback<bool>),
    CloseWebView {
        webview_id: WebViewId,
        prompt_unload: bool,
        callback: GenericCallback<bool>,
    },
    Request(String),
    TraverseHistory(BrowsingContextId, i64, GenericCallback<bool>),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum WebDriverToScriptMessage {
    Reload,
    // bool is prompt unload
    CloseNavigable(bool, GenericCallback<()>),
}
