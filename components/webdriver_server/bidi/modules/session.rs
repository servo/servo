use std::rc::Rc;

use webdriver_traits::bidi::{SessionCommand, SessionResult};

use crate::bidi::{
    error::{BidiError, BidiResult},
    remote_end::RemoteEnd,
};

impl RemoteEnd {
    pub(crate) async fn handle_session_command(
        self: Rc<Self>,
        command: SessionCommand,
    ) -> BidiResult<SessionResult> {
        // TODO:
        Err(BidiError::default())
    }
}
