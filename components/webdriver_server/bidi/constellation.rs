use std::rc::Rc;

use webdriver_traits::ConstellationToWebDriverMsg;

use crate::bidi::remote_end::RemoteEnd;

impl RemoteEnd {
    pub(crate) fn handle_constellation(self: Rc<Self>, msg: ConstellationToWebDriverMsg) {
        match msg {
            ConstellationToWebDriverMsg::ContextCreated(info) => self.handle_context_created(),
            ConstellationToWebDriverMsg::ContextDestroyed(info) => self.handle_context_destroyed(),
            ConstellationToWebDriverMsg::DomContentLoaded(navigation_info) => {
                self.handle_dom_content_loaded()
            },
            ConstellationToWebDriverMsg::DownloadWillBegin(download_will_begin_params) => {
                self.handle_download_will_begin()
            },
            ConstellationToWebDriverMsg::DownloadEnd(download_end_params) => {
                self.handle_download_end()
            },
            ConstellationToWebDriverMsg::FragmentNavigated(navigation_info) => {
                self.handle_fragment_navigated()
            },
            ConstellationToWebDriverMsg::HistoryUpdated(history_updated_parameters) => {
                self.handle_history_updated()
            },
            ConstellationToWebDriverMsg::Load(navigation_info) => self.handle_load(),
            ConstellationToWebDriverMsg::NavigationStarted(navigation_info) => {
                self.handle_navigation_started()
            },
            ConstellationToWebDriverMsg::NavigationAborted(navigation_info) => {
                self.handle_navigation_aborted()
            },
            ConstellationToWebDriverMsg::NavigationCommitted(navigation_info) => {
                self.handle_navigation_committed()
            },
            ConstellationToWebDriverMsg::NavigationFailed(navigation_info) => {
                self.handle_navigation_failed()
            },
        }
    }

    fn handle_context_created(self: Rc<Self>) {
        // TODO: record context hierarchy
        // TODO:
        todo!()
    }

    fn handle_context_destroyed(self: Rc<Self>) {
        todo!()
    }

    fn handle_dom_content_loaded(self: Rc<Self>) {
        todo!()
    }

    fn handle_download_will_begin(self: Rc<Self>) {
        todo!()
    }

    fn handle_download_end(self: Rc<Self>) {
        todo!()
    }

    fn handle_fragment_navigated(self: Rc<Self>) {
        todo!()
    }

    fn handle_history_updated(self: Rc<Self>) {
        todo!()
    }

    fn handle_load(self: Rc<Self>) {
        todo!()
    }

    fn handle_navigation_started(self: Rc<Self>) {
        todo!()
    }

    fn handle_navigation_aborted(self: Rc<Self>) {
        todo!()
    }

    fn handle_navigation_committed(self: Rc<Self>) {
        todo!()
    }

    fn handle_navigation_failed(self: Rc<Self>) {
        todo!()
    }
}
