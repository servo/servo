use serde::{Deserialize, Serialize};
use servo_base::{
    generic_channel::GenericSender,
    id::{BrowsingContextId, WebViewId},
};
use url::Url;

// TODO: check traits impl of other id types in id.rs

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionId(u32);

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionId(u32);

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandId(u32);

#[derive(Debug, Serialize, Deserialize)]
pub enum RequestId {
    /// When command is static, need to know which connection to associate.
    Static(ConnectionId),
    /// Session may have multiple connections and duplicate command ids,
    /// but that does not matter as we can send all message back to every
    /// connection.
    Session(SessionId, CommandId),
}

// NOTE: the handler does not need to know which request send the command
// so RequestId can better be bind to sender, not separately.
#[derive(Debug, Deserialize, Serialize)]
pub struct RequestSender<T: Serialize> {
    pub id: RequestId,
    pub sender: GenericSender<T>,
}

/// Messages to the constellation originating from the WebDriver BiDi server.
// TODO: incomplete
#[derive(Debug)]
pub enum WebDriverBidiCommandMsg {
    TraverseHistory(WebViewId, RequestSender<()>),
    Navigate(WebViewId, Url, RequestSender<()>),
    Reload(WebViewId, Url),
    /// Pass a webdriver bidi command to the script thread of the current pipeline
    /// of a browsing context.
    ScriptCommand(BrowsingContextId, WebDriverBidiScriptCommand),
}

/// Commands sent to the content process.
// TODO: incomplete
#[derive(Debug, Deserialize, Serialize)]
pub enum WebDriverBidiScriptCommand {
    DeleteCookies(RequestSender<()>),
}
