use std::rc::Rc;

use webdriver_traits::bidi::{BrowsingContextCommand, BrowsingContextResult};

use crate::bidi::{
    error::{BidiError, BidiResult},
    remote_end::RemoteEnd,
};

impl RemoteEnd {
    pub(crate) async fn handle_browsing_context_command(
        self: Rc<Self>,
        command: BrowsingContextCommand,
    ) -> BidiResult<BrowsingContextResult> {
        // TODO:
        Err(BidiError::default())
    }
}
