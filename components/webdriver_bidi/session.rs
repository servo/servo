use serde::{Deserialize, Serialize};
use servo_base::id::{BrowsingContextId, WebViewId};
use uuid::Uuid;

use crate::{handler::WebDriverBidiHandler, connection::Connection};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SessionId(Uuid);

pub(crate) struct WebDriverBidiSession {
    id: Uuid,
    webview_id: Option<WebViewId>,
    browsing_context_id: Option<BrowsingContextId>,
}

pub struct Session<T: WebDriverBidiHandler> {
    connections: Vec<Connection>,
    handler: T,
}
