use std::sync::atomic::{AtomicU64, Ordering::SeqCst};

use serde::{Deserialize, Serialize};
use servo_base::{
    generic_channel::GenericSender,
    id::{BrowsingContextId, WebViewId},
};
use url::Url;

// TODO: check traits impl of other id types in id.rs

static REQUEST_ID: AtomicU64 = AtomicU64::new(0);

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
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize)]
pub struct RequestId(u64);

impl From<u64> for RequestId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl RequestId {
    pub fn next() -> Self {
        Self(REQUEST_ID.fetch_add(1, SeqCst))
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
pub enum WebDriverBidiToEmbedderMsg {
    TraverseHistory(WebViewId, i64),
    Navigate(WebViewId, Url, RequestSender<()>),
    Shutdown,
    // TODO: wait state should be enum
    // TODO: use named fields
    BrowsingContextReload(BrowsingContextId, bool, WaitCondition),
    /// Pass a webdriver bidi command to the script thread of the current pipeline
    /// of a browsing context.
    ScriptCommand(BrowsingContextId, WebDriverBidiToScriptMsg),
}

/// Commands sent to the content process.
// TODO: incomplete
#[derive(Debug, Deserialize, Serialize)]
pub enum WebDriverBidiToScriptMsg {
    DeleteCookies(RequestSender<()>),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WaitCondition {
    Committed,
    Interactive,
    Complete,
}
