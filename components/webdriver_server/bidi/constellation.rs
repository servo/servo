use webdriver_traits::ConstellationToWebDriverMsg;

use crate::bidi::remote_end::RemoteEnd;

impl RemoteEnd {
    pub(crate) fn handle_constellation(&self, msg: ConstellationToWebDriverMsg) {
        match msg {
            ConstellationToWebDriverMsg::BrowsingContextCreated(info) => {
                // TODO: is this really needed
                // what info do we need
            },
        }
    }
}
