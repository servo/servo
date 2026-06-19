use std::rc::Rc;

use webdriver_traits::bidi::{InputCommand, InputResult};

use crate::bidi::{
    error::{BidiError, BidiResult},
    remote_end::RemoteEnd,
};

impl RemoteEnd {
    pub(crate) async fn handle_input_command(
        self: Rc<Self>,
        command: InputCommand,
    ) -> BidiResult<InputResult> {
        // TODO:
        Err(BidiError::default())
    }
}
