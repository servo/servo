/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::id::{BrowsingContextId, PipelineId, WebViewId};
use constellation_traits::ScriptToConstellationMessage;
use ipc_channel::ipc;
use rustc_hash::FxBuildHasher;
use script_bindings::inheritance::Castable;
use script_bindings::root::{Dom, DomRoot};
use script_bindings::str::DOMString;

use crate::document_collection::DocumentCollection;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::trace::HashMapTracedValues;
use crate::dom::node::NodeTraits;
use crate::dom::types::{GlobalScope, Window};
use crate::dom::windowproxy::{CreatorBrowsingContextInfo, WindowProxy};
use crate::messaging::ScriptThreadSenders;

#[derive(JSTraceable, Default, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::allow_unrooted_in_rc)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct ScriptWindowProxies {
    map: DomRefCell<HashMapTracedValues<BrowsingContextId, Dom<WindowProxy>, FxBuildHasher>>,
}

impl ScriptWindowProxies {
    pub(crate) fn find_window_proxy(&self, id: BrowsingContextId) -> Option<DomRoot<WindowProxy>> {
        self.map
            .borrow()
            .get(&id)
            .map(|context| DomRoot::from_ref(&**context))
    }

    pub(crate) fn find_window_proxy_by_name(
        &self,
        name: &DOMString,
    ) -> Option<DomRoot<WindowProxy>> {
        for (_, proxy) in self.map.borrow().iter() {
            if proxy.get_name() == *name {
                return Some(DomRoot::from_ref(&**proxy));
            }
        }
        None
    }

    pub(crate) fn get(&self, id: BrowsingContextId) -> Option<DomRoot<WindowProxy>> {
        self.map
            .borrow()
            .get(&id)
            .map(|context| DomRoot::from_ref(&**context))
    }

    pub(crate) fn insert(&self, id: BrowsingContextId, proxy: DomRoot<WindowProxy>) {
        self.map.borrow_mut().insert(id, Dom::from_ref(&*proxy));
    }

    pub(crate) fn remove(&self, id: BrowsingContextId) {
        self.map.borrow_mut().remove(&id);
    }

    // Get the browsing context for a pipeline that may exist in another
    // script thread.  If the browsing context already exists in the
    // `window_proxies` map, we return it, otherwise we recursively
    // get the browsing context for the parent if there is one,
    // construct a new dissimilar-origin browsing context, add it
    // to the `window_proxies` map, and return it.
    pub(crate) fn remote_window_proxy(
        &self,
        senders: &ScriptThreadSenders,
        global_to_clone: &GlobalScope,
        webview_id: WebViewId,
        pipeline_id: PipelineId,
        opener: Option<BrowsingContextId>,
    ) -> Option<DomRoot<WindowProxy>> {
        let (browsing_context_id, parent_pipeline_id) =
            self.ask_constellation_for_browsing_context_info(senders, webview_id, pipeline_id)?;
        if let Some(window_proxy) = self.get(browsing_context_id) {
            return Some(window_proxy);
        }

        let parent_browsing_context = parent_pipeline_id.and_then(|parent_id| {
            self.remote_window_proxy(senders, global_to_clone, webview_id, parent_id, opener)
        });

        let opener_browsing_context = opener.and_then(|id| self.find_window_proxy(id));

        let creator = CreatorBrowsingContextInfo::from(
            parent_browsing_context.as_deref(),
            opener_browsing_context.as_deref(),
        );

        let window_proxy = WindowProxy::new_dissimilar_origin(
            global_to_clone,
            browsing_context_id,
            webview_id,
            parent_browsing_context.as_deref(),
            opener,
            creator,
        );
        self.insert(browsing_context_id, DomRoot::from_ref(&*window_proxy));
        Some(window_proxy)
    }

    // Get the browsing context for a pipeline that exists in this
    // script thread.  If the browsing context already exists in the
    // `window_proxies` map, we return it, otherwise we recursively
    // get the browsing context for the parent if there is one,
    // construct a new similar-origin browsing context, add it
    // to the `window_proxies` map, and return it.
    #[expect(clippy::too_many_arguments)]
    pub(crate) fn local_window_proxy(
        &self,
        senders: &ScriptThreadSenders,
        documents: &DomRefCell<DocumentCollection>,
        window: &Window,
        browsing_context_id: BrowsingContextId,
        webview_id: WebViewId,
        parent_info: Option<PipelineId>,
        opener: Option<BrowsingContextId>,
    ) -> DomRoot<WindowProxy> {
        if let Some(window_proxy) = self.get(browsing_context_id) {
            // Note: we do not set the window to be the currently-active one,
            // this will be done instead when the script-thread handles the `SetDocumentActivity` msg.
            return window_proxy;
        }
        let iframe = parent_info.and_then(|parent_id| {
            documents
                .borrow()
                .find_iframe(parent_id, browsing_context_id)
        });
        let parent_browsing_context = match (parent_info, iframe.as_ref()) {
            (_, Some(iframe)) => Some(iframe.owner_window().window_proxy()),
            (Some(parent_id), _) => {
                self.remote_window_proxy(senders, window.upcast(), webview_id, parent_id, opener)
            },
            _ => None,
        };

        let opener_browsing_context = opener.and_then(|id| self.find_window_proxy(id));

        let creator = CreatorBrowsingContextInfo::from(
            parent_browsing_context.as_deref(),
            opener_browsing_context.as_deref(),
        );

        let window_proxy = WindowProxy::new(
            window,
            browsing_context_id,
            webview_id,
            iframe.as_deref().map(Castable::upcast),
            parent_browsing_context.as_deref(),
            opener,
            creator,
        );
        self.insert(browsing_context_id, DomRoot::from_ref(&*window_proxy));
        window_proxy
    }

    fn ask_constellation_for_browsing_context_info(
        &self,
        senders: &ScriptThreadSenders,
        webview_id: WebViewId,
        pipeline_id: PipelineId,
    ) -> Option<(BrowsingContextId, Option<PipelineId>)> {
        let (result_sender, result_receiver) = ipc::channel().unwrap();
        let msg = ScriptToConstellationMessage::GetBrowsingContextInfo(pipeline_id, result_sender);
        senders
            .pipeline_to_constellation_sender
            .send((webview_id, pipeline_id, msg))
            .expect("Failed to send to constellation.");
        result_receiver
            .recv()
            .expect("Failed to get browsing context info from constellation.")
    }
}
