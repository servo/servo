use std::rc::Rc;

use webdriver_traits::bidi::{EmulationCommand, EmulationResult};

use crate::bidi::{
    error::{BidiError, BidiResult},
    remote_end::RemoteEnd,
};

impl RemoteEnd {
    pub(crate) async fn handle_emulation_command(
        self: Rc<Self>,
        command: EmulationCommand,
    ) -> BidiResult<EmulationResult> {
        // TODO:
        Err(BidiError::default())
    }
}
