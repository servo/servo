use std::rc::Rc;

use webdriver_traits::bidi::{BrowserCommand, BrowserResult};

use crate::bidi::{
    error::{BidiError, BidiResult},
    remote_end::RemoteEnd,
};

impl RemoteEnd {
    pub(crate) async fn handle_browser_command(
        self: Rc<Self>,
        command: BrowserCommand,
    ) -> BidiResult<BrowserResult> {
        // TODO:
        Err(BidiError::default())
    }
}
