use webdriver_traits::ConstellationToWebDriverMsg;

use crate::bidi::remote_end::RemoteEnd;

impl RemoteEnd {
    pub(crate) fn handle_constellation(&self, msg: ConstellationToWebDriverMsg) {
        match msg {
            ConstellationToWebDriverMsg::ContextCreated(info) => {
                // TODO: is this really needed
                // what info do we need
            },
            ConstellationToWebDriverMsg::ContextDestroyed(info) => todo!(),
            ConstellationToWebDriverMsg::DomContentLoaded(navigation_info) => todo!(),
            ConstellationToWebDriverMsg::DownloadWillBegin(download_will_begin_params) => todo!(),
            ConstellationToWebDriverMsg::DownloadEnd(download_end_params) => todo!(),
            ConstellationToWebDriverMsg::FragmentNavigated(navigation_info) => todo!(),
            ConstellationToWebDriverMsg::HistoryUpdated(history_updated_parameters) => todo!(),
            ConstellationToWebDriverMsg::Load(navigation_info) => todo!(),
            ConstellationToWebDriverMsg::NavigationStarted(navigation_info) => todo!(),
            ConstellationToWebDriverMsg::NavigationAborted(navigation_info) => todo!(),
            ConstellationToWebDriverMsg::NavigationCommitted(navigation_info) => todo!(),
            ConstellationToWebDriverMsg::NavigationFailed(navigation_info) => todo!(),
        }
    }
}
