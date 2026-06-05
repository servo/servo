use std::rc::Rc;

use log::warn;
use servo::webdriver_bidi::WebDriverBidiCommandMsg;

use crate::running_app_state::RunningAppState;

impl RunningAppState {
    pub(crate) fn handle_webdriver_bidi_message(self: &Rc<Self>) {
        let Some(webdriver_bidi_receiver) = self.webdriver_bidi_receiver() else {
            return;
        };

        while let Ok(msg) = webdriver_bidi_receiver.try_recv() {
            // TODO: match and handle
            match msg {
                // NOTE: as a reference for others
                WebDriverBidiCommandMsg::TraverseHistory(web_view_id, delta) => {
                    if let Some(webview) = self.webview_by_id(web_view_id) {
                        // BiDi does not support wait for traverseHistory, see Step 10.
                        // TODO: set sender may still be needed for events?
                        // Not sure, event is not bound to a command, and others
                        // can also trigger events
                        // TODO: follow and document the steps
                        match delta {
                            1.. => {
                                webview.go_forward(delta.unsigned_abs() as usize);
                            },
                            ..0 => {
                                webview.go_back(delta.unsigned_abs() as usize);
                            },
                            0 => {},
                        }
                    }
                },
                WebDriverBidiCommandMsg::Navigate(web_view_id, url, request_sender) => todo!(),
                // WebDriverBidiCommandMsg::Reload(web_view_id) => {
                //     if let Some(webview) = self.webview_by_id(web_view_id) {
                //         // self.set_load_status_sender(webview_id, sender);
                //         webview.reload();
                //     }
                // },
                WebDriverBidiCommandMsg::ScriptCommand(
                    namespace_index,
                    web_driver_bidi_script_command,
                ) => todo!(),
                WebDriverBidiCommandMsg::Shutdown => {
                    self.schedule_exit();
                },
                WebDriverBidiCommandMsg::BrowsingContextReload(
                    namespace_index,
                    _,
                    wait_condition,
                ) => todo!(),
            }
        }
    }
}
