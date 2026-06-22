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

    #[allow(clippy::derivable_impls)]
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

    #[allow(clippy::derivable_impls)]
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

pub mod ids;

use devtools_traits::WorkerId;
use serde::{Deserialize, Serialize};
use servo_base::{
    generic_channel::{GenericCallback, GenericSender},
    id::{BrowsingContextId, PainterId, PipelineId, WebViewId},
};

use crate::bidi::{
    ErrorCode,
    browser::SetClientWindowStateParameters,
    browsing_context::{
        self, ClipRectangle, CreateType, DownloadEndParams, DownloadWillBeginParams,
        HistoryUpdatedParameters, Locator, NavigationInfo, PrintParameters, UserPromptClosed,
        UserPromptClosedParameters, UserPromptOpenedParameters,
    },
    emulation::{
        ForcedColorsModeTheme, ScreenArea, ScreenOrientation, SetGeolocationOverrideParameters,
        SetScrollbarTypeOverrideParametersScrollbarType,
    },
    input, log,
    script::{
        self, AddPreloadScriptParameters, CallFunctionParameters, DisownParameters,
        EvaluateParameters, EvaluateResult, NodeRemoteValue, SerializationOptions,
    },
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
    ContextCreated(browsing_context::Info),
    ContextDestroyed(browsing_context::Info),
    DomContentLoaded(NavigationInfo),
    DownloadWillBegin(DownloadWillBeginParams),
    DownloadEnd(DownloadEndParams),
    FragmentNavigated(NavigationInfo),
    HistoryUpdated(HistoryUpdatedParameters),
    Load(NavigationInfo),
    NavigationStarted(NavigationInfo),
    NavigationAborted(NavigationInfo),
    NavigationCommitted(NavigationInfo),
    NavigationFailed(NavigationInfo),
}

// TODO: command responses need session id
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ScriptToWebDriverMsg {
    LogEntryAddedConsole(Vec<BrowsingContextId>, log::EntryAdded),
    LogEntryAddedErrorReporting(Vec<BrowsingContextId>, log::EntryAdded),
    RealmCreated(
        (BrowsingContextId, PipelineId, Option<WorkerId>, WebViewId),
        GenericSender<WebDriverToScriptMsg>,
    ),
    RealmDestroyed(PipelineId, Option<WorkerId>),
    /// When a channel previously sent to script thread is called.
    ChannelMessage {
        // TODO: channel should have more speicific id type
        channel: String,
        data: script::RemoteValue, // TODO: source with realm id & context id
    },
    FileDialogOpened(input::FileDialogOpened),
    UserPromptClosed(UserPromptClosedParameters),
    UserPromptOpened(UserPromptOpenedParameters),
}

// TODO: remove all callback

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum WebDriverToConstellationMsg {
    AddPreloadScript(AddPreloadScriptParameters, GenericCallback<String>),
    RemovePreloadScript(String, GenericCallback<String>),
    SetBypassCsp(WebViewId, GenericCallback<()>),
    SetForcedColorsModeOverride(
        WebViewId,
        Option<ForcedColorsModeTheme>,
        GenericCallback<()>,
    ),
    SetGeolocationOverride(
        WebViewId,
        SetGeolocationOverrideParameters,
        GenericCallback<()>,
    ),
    SetLocaleOverride(WebViewId, Option<String>, GenericCallback<()>),
    SetScreenSettingsOverride(WebViewId, Option<ScreenArea>),
    SetScreenOrientationOverride(WebViewId, Option<ScreenOrientation>),
    SetScriptEnabledOverride(WebViewId, Option<String>),
    SetScrollbarTypeOverride(
        WebViewId,
        Option<SetScrollbarTypeOverrideParametersScrollbarType>,
    ),
    SetTimezoneOverride(WebViewId, Option<String>),
    SetTouchOverride(WebViewId, Option<u64>),
    SetUserAgentOverride(WebViewId, Option<String>),
    SetViewport(WebViewId, GenericCallback<()>),
    TraverseHistory(WebViewId, i64, GenericCallback<bool>),
    Close(WebViewId, bool, GenericCallback<bool>),
    Navigate(WebViewId, String, GenericCallback<()>),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum WebDriverToScriptMsg {
    CallFunction(CallFunctionParameters, GenericCallback<EvaluateResult>),
    CaptureScreenshot(BrowsingContextId, ClipRectangle, GenericCallback<String>),
    Disown(DisownParameters, GenericCallback<()>),
    Evaluate(EvaluateParameters, GenericCallback<EvaluateResult>),
    HandleUserPrompt(BrowsingContextId, Option<bool>, Option<String>),
    // TODO: startNodes
    LocateNode(
        BrowsingContextId,
        Locator,
        Option<u64>,
        SerializationOptions,
        GenericCallback<Vec<NodeRemoteValue>>,
    ),
    Print(PrintParameters, GenericCallback<String>),
    Reload(bool, GenericCallback<()>),
    StartScreencast(BrowsingContextId, GenericCallback<()>),
    StopScreencast(BrowsingContextId, GenericCallback<()>),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum WebDriverToEmbedderMsg {
    Activate(WebViewId, GenericCallback<bool>),
    Exit,
    // TODO: param
    SetClientWindowState(PainterId, SetClientWindowStateParameters),
    WebViewCreate(WebViewCreateRequest),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebViewCreateRequest {
    pub create_type: CreateType,
    pub opener: Option<WebViewId>,
    pub callback: GenericCallback<Result<WebViewId, ErrorCode>>,
}
