use std::rc::Rc;

use webdriver_traits::bidi::{ScriptCommand, ScriptResult};

use crate::bidi::{
    error::{BidiError, BidiResult},
    remote_end::RemoteEnd,
};

impl RemoteEnd {
    pub(crate) async fn handle_script_command(
        self: Rc<Self>,
        command: ScriptCommand,
    ) -> BidiResult<ScriptResult> {
        // TODO:
        Err(BidiError::default())
    }
}
