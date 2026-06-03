use std::rc::Rc;

use crate::running_app_state::RunningAppState;

impl RunningAppState {
    pub(crate) fn handle_webdriver_bidi_message(self: &Rc<Self>) {
        let Some(webdriver_bidi_receiver) = self.webdriver_bidi_receiver() else {
            return;
        };

        while let Ok(msg) = webdriver_bidi_receiver.try_recv() {
            // TODO: match and handle
        }
    }
}
