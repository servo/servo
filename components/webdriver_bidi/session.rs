use servo_base::id::{BrowsingContextId, WebViewId};
use uuid::Uuid;

pub(crate) struct WebDriverBidiSession {
    id: Uuid,
    webview_id: Option<WebViewId>,
    browsing_context_id: Option<BrowsingContextId>,
}
