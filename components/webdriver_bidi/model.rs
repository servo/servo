//! We basically use extern crate rustenium_bidi_definitions for WebDriver BiDi
//! definitions. But some extra definitions are required: e.g. rustenium does
//! not define enum of ResultData and custom commands.

use rustenium_bidi_definitions::{
    base::{ErrorResponse, EventResponse, Extensible, SuccessEnum},
    browser::results as browser,
    browsing_context::results as browsing_context,
    emulation::results as emulation,
    input::results as input,
    network::results as network,
    script::results as script,
    session::results as session,
    storage::results as storage,
    web_extension::results as web_extension,
};
use serde::Serialize;

/// Similar to `Message` in `rustenium` but using concrete enum instead of JSON.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Message {
    ErrorResponse(ErrorResponse),
    CommandResponse(Box<CommandResponse>),
    Event(Box<EventResponse>),
}

/// Similar to `CommandResponse` in `rustenium` but using concrete enum instead
/// of JSON.
#[derive(Debug, Clone, Serialize)]
pub struct CommandResponse {
    pub r#type: SuccessEnum,
    pub id: u64,
    pub result: ResultData,
    pub extensible: Extensible,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ResultData {
    Browser(BrowserResult),
    BrowsingContext(BrowsingContextResult),
    Emulation(EmulationResult),
    Input(InputResult),
    Network(NetworkResult),
    Script(ScriptResult),
    Session(SessionResult),
    Storage(StorageResult),
    WebExtension(WebExtensionResult),
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum BrowserResult {
    Close(browser::CloseResult),
    CreateUserContext(browser::CreateUserContextResult),
    GetClientWindows(browser::GetClientWindowsResult),
    GetUserContexts(browser::GetUserContextsResult),
    RemoveUserContext(browser::RemoveUserContextResult),
    SetClientWindowState(browser::SetClientWindowStateResult),
    SetDownloadBehavior(browser::SetDownloadBehaviorResult),
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum BrowsingContextResult {
    Activate(browsing_context::ActivateResult),
    CaptureScreenshot(browsing_context::CaptureScreenshotResult),
    Close(browsing_context::CloseResult),
    Create(browsing_context::CreateResult),
    GetTree(browsing_context::GetTreeResult),
    HandleUserPrompt(browsing_context::HandleUserPromptResult),
    LocateNodes(browsing_context::LocateNodesResult),
    Navigate(browsing_context::NavigateResult),
    Print(browsing_context::PrintResult),
    Reload(browsing_context::ReloadResult),
    SetViewport(browsing_context::SetViewportResult),
    TraverseHistory(browsing_context::TraverseHistoryResult),
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum EmulationResult {
    SetForcedColorsModeThemeOverride(emulation::SetForcedColorsModeThemeOverrideResult),
    SetGeolocationOverride(emulation::SetGeolocationOverrideResult),
    SetLocaleOverride(emulation::SetLocaleOverrideResult),
    SetNetworkConditions(emulation::SetNetworkConditionsResult),
    SetScreenOrientationOverride(emulation::SetScreenOrientationOverrideResult),
    SetUserAgentOverride(emulation::SetUserAgentOverrideResult),
    SetScriptingEnabled(emulation::SetScriptingEnabledResult),
    SetTimezoneOverride(emulation::SetTimezoneOverrideResult),
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum InputResult {
    PerformActions(input::PerformActionsResult),
    ReleaseActions(input::ReleaseActionsResult),
    SetFiles(input::SetFilesResult),
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum NetworkResult {
    AddDataCollector(network::AddDataCollectorResult),
    AddIntercept(network::AddInterceptResult),
    ContinueRequest(network::ContinueRequestResult),
    ContinueResponse(network::ContinueResponseResult),
    ContinueWithAuth(network::ContinueWithAuthResult),
    DisownData(network::DisownDataResult),
    FailRequest(network::FailRequestResult),
    GetData(network::GetDataResult),
    ProvideResponse(network::ProvideResponseResult),
    RemoveDataCollector(network::RemoveDataCollectorResult),
    RemoveIntercept(network::RemoveInterceptResult),
    SetCacheBehavior(network::SetCacheBehaviorResult),
    SetExtraHeaders(network::SetExtraHeadersResult),
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ScriptResult {
    AddPreloadScript(script::AddPreloadScriptResult),
    Disown(script::DisownResult),
    CallFunction(script::CallFunctionResult),
    Evaluate(script::EvaluateResult),
    GetRealms(script::GetRealmsResult),
    RemovePreloadScript(script::RemovePreloadScriptResult),
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum SessionResult {
    Status(session::StatusResult),
    New(Box<session::NewResult>),
    End(session::EndResult),
    Subscribe(session::SubscribeResult),
    Unsubscribe(session::UnsubscribeResult),
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum StorageResult {
    GetCookies(storage::GetCookiesResult),
    SetCookie(storage::SetCookieResult),
    DeleteCookies(storage::DeleteCookiesResult),
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum WebExtensionResult {
    Install(web_extension::InstallResult),
    Uninstall(web_extension::UninstallResult),
}
