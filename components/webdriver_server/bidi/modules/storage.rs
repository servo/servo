use std::rc::Rc;

use webdriver_traits::bidi::{StorageCommand, StorageResult};

use crate::bidi::{
    error::{BidiError, BidiResult},
    remote_end::RemoteEnd,
};

impl RemoteEnd {
    pub(crate) async fn handle_storage_command(
        self: Rc<Self>,
        command: StorageCommand,
    ) -> BidiResult<StorageResult> {
        // TODO:
        Err(BidiError::default())
    }
}
