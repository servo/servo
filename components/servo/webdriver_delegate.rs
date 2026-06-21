use std::rc::Rc;

use profile_traits::generic_callback::GenericCallback;
use servo_base::id::{PainterId, WebViewId};
use url::Url;
pub use webdriver_traits::{
    WebViewCreateRequest,
    bidi::{ErrorCode, browsing_context::CreateType},
};

pub trait WebDriverDelegate {
    // TODO; notify started instead of log

    /// Whether the implementation supports multiple top-level traversable in separate OS windows.
    fn support_multiple_window(&self) -> bool {
        false
    }

    /// We need to save the requests to a queue because OS window factory is
    /// not available in servo spin loop. Embedder should handle these requests later.
    fn queue_request(&self, request: WebViewCreateRequest) {}
}

pub struct DefaultWebDriverDelegate;
impl WebDriverDelegate for DefaultWebDriverDelegate {}
