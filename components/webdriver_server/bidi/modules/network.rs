use std::rc::Rc;

use webdriver_traits::bidi::{NetworkCommand, NetworkResult};

use crate::bidi::{
    error::{BidiError, BidiResult},
    remote_end::RemoteEnd,
};

impl RemoteEnd {
    pub(crate) async fn handle_network_command(
        self: Rc<Self>,
        command: NetworkCommand,
    ) -> BidiResult<NetworkResult> {
        // TODO:
        Err(BidiError::default())
    }
}
