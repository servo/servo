use serde::{Deserialize, Serialize};
use servo_base::{
    generic_channel::GenericSender,
    id::{BrowsingContextId, WebViewId},
};
use url::Url;

// TODO: check traits impl of other id types in id.rs

/// The internal request id for every BiDi command.
///
/// ## FAQ
///
/// ### Why do we need keep id internally?
///
/// BiDi clients can send duplicate command id. This happens
/// mostly when multiple client connects to same endpoint and
/// send `session.new`, but it can also happen when client does
/// not correctly increase the id.
///
/// This can also abstract away connection and session detail from the
/// embedder.
///
/// ### Why named `RequestId`?
///
/// To distinguish with [`CommandResponse`]'s internal id. Also
/// this mechanism may be reused to handle classic WebDriver
/// reqeusts later when these two impls are merged.
// TODO: be opaque
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash, Default)]
pub struct RequestId(pub u64);

// TODO: refactor according to other id
impl RequestId {
    pub fn inc(&mut self) -> Self {
        let old = *self;
        self.0 += 1;
        old
    }
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
    TraverseHistory(WebViewId, i64),
    Navigate(WebViewId, Url, RequestSender<()>),
    Reload(WebViewId),
    Shutdown,
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
