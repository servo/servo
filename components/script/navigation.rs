/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The listener that encapsulates all state for an in-progress document request.
//! Any redirects that are encountered are followed. Whenever a non-redirect
//! response is received, it is forwarded to the appropriate script thread.

use std::cell::Cell;

use content_security_policy::sandboxing_directive::SandboxingFlagSet;
use crossbeam_channel::Sender;
use embedder_traits::user_contents::UserContentManagerId;
use embedder_traits::{Theme, ViewportDetails, WebDriverLoadStatus};
use http::header;
use js::context::JSContext;
use net_traits::blob_url_store::UrlWithBlobClaim;
use net_traits::policy_container::RequestPolicyContainer;
use net_traits::request::{
    CredentialsMode, InsecureRequestsPolicy, Origin, PreloadedResources, RedirectMode,
    RequestBuilder, RequestClient, RequestMode,
};
use net_traits::response::ResponseInit;
use net_traits::{
    BoxedFetchCallback, CoreResourceThread, DOCUMENT_ACCEPT_HEADER_VALUE, FetchResponseMsg,
    Metadata, ReferrerPolicy, fetch_async, set_default_accept_language,
};
use script_bindings::inheritance::Castable;
use script_traits::{DocumentActivity, NewPipelineInfo};
use servo_base::cross_process_instant::CrossProcessInstant;
use servo_base::id::{BrowsingContextId, PipelineId, WebViewId};
use servo_constellation_traits::{
    LoadData, LoadOrigin, NavigationHistoryBehavior, ScriptToConstellationMessage,
    TargetSnapshotParams,
};
use servo_url::{ImmutableOrigin, MutableOrigin, ServoUrl};
use url::Position;

use crate::dom::bindings::codegen::Bindings::HTMLIFrameElementBinding::HTMLIFrameElementMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::element::Element;
use crate::dom::html::htmliframeelement::HTMLIFrameElement;
use crate::dom::node::node::NodeTraits;
use crate::dom::window::Window;
use crate::dom::windowproxy::WindowProxy;
use crate::fetch::FetchCanceller;
use crate::messaging::MainThreadScriptMsg;
use crate::script_runtime::CanGc;
use crate::script_thread::ScriptThread;

#[derive(Clone)]
pub struct NavigationListener {
    request_builder: RequestBuilder,
    main_thread_sender: Sender<MainThreadScriptMsg>,
    // Whether or not results are sent to the main thread. After a redirect results are no longer sent,
    // as the main thread has already started a new request.
    send_results_to_main_thread: Cell<bool>,
}

impl NavigationListener {
    pub(crate) fn into_callback(self) -> BoxedFetchCallback {
        Box::new(move |response_msg| self.notify_fetch(response_msg))
    }

    pub fn new(
        request_builder: RequestBuilder,
        main_thread_sender: Sender<MainThreadScriptMsg>,
    ) -> NavigationListener {
        NavigationListener {
            request_builder,
            main_thread_sender,
            send_results_to_main_thread: Cell::new(true),
        }
    }

    pub fn initiate_fetch(
        self,
        core_resource_thread: &CoreResourceThread,
        response_init: Option<ResponseInit>,
    ) {
        fetch_async(
            core_resource_thread,
            self.request_builder.clone(),
            response_init,
            self.into_callback(),
        );
    }

    fn notify_fetch(&self, message: FetchResponseMsg) {
        // If we've already asked the main thread to redirect the response, then stop sending results
        // for this fetch. The main thread has already replaced it.
        if !self.send_results_to_main_thread.get() {
            return;
        }

        // If this is a redirect, don't send any more message after this one.
        if Self::http_redirect_metadata(&message).is_some() {
            self.send_results_to_main_thread.set(false);
        }

        let pipeline_id = self
            .request_builder
            .pipeline_id
            .expect("Navigation should always have an associated Pipeline");
        let result = self
            .main_thread_sender
            .send(MainThreadScriptMsg::NavigationResponse {
                pipeline_id,
                message: Box::new(message),
            });

        if let Err(error) = result {
            warn!(
                "Failed to send network message to pipeline {:?}: {error:?}",
                pipeline_id
            );
        }
    }

    pub(crate) fn http_redirect_metadata(message: &FetchResponseMsg) -> Option<&Metadata> {
        let FetchResponseMsg::ProcessResponse(_, Ok(metadata)) = message else {
            return None;
        };

        // Don't allow redirects for non HTTP(S) URLs.
        let metadata = metadata.metadata();
        if !matches!(
            metadata.location_url,
            Some(Ok(ref location_url)) if matches!(location_url.scheme(), "http" | "https")
        ) {
            return None;
        }

        Some(metadata)
    }
}

/// A document load that is in the process of fetching the requested resource. Contains
/// data that will need to be present when the document and frame tree entry are created,
/// but is only easily available at initiation of the load and on a push basis (so some
/// data will be updated according to future resize events, viewport changes, etc.)
#[derive(JSTraceable)]
pub(crate) struct InProgressLoad {
    /// The pipeline which requested this load.
    #[no_trace]
    pub(crate) pipeline_id: PipelineId,
    /// The browsing context being loaded into.
    #[no_trace]
    pub(crate) browsing_context_id: BrowsingContextId,
    /// The top level ancestor browsing context.
    #[no_trace]
    pub(crate) webview_id: WebViewId,
    /// The parent pipeline and frame type associated with this load, if any.
    #[no_trace]
    pub(crate) parent_info: Option<PipelineId>,
    /// The opener, if this is an auxiliary.
    #[no_trace]
    pub(crate) opener: Option<BrowsingContextId>,
    /// The current window size associated with this pipeline.
    #[no_trace]
    pub(crate) viewport_details: ViewportDetails,
    /// The activity level of the document (inactive, active or fully active).
    #[no_trace]
    pub(crate) activity: DocumentActivity,
    /// Window is throttled, running timers at a heavily limited rate.
    pub(crate) throttled: bool,
    /// Timestamp reporting the time when the browser started this load.
    #[no_trace]
    pub(crate) navigation_start: CrossProcessInstant,
    /// For cancelling the fetch
    pub(crate) canceller: FetchCanceller,
    /// The [`LoadData`] associated with this load.
    #[no_trace]
    pub(crate) load_data: LoadData,
    /// A list of URL to keep track of all the redirects that have happened during
    /// this load.
    #[no_trace]
    pub(crate) url_list: Vec<ServoUrl>,
    #[no_trace]
    /// The [`UserContentManagerId`] associated with this load's `WebView`.
    pub(crate) user_content_manager_id: Option<UserContentManagerId>,
    /// The [`Theme`] to use for this page, once it loads.
    #[no_trace]
    pub(crate) theme: Theme,
    /// The [`TargetSnapshotParams`] to use when creating this document.
    #[no_trace]
    pub(crate) target_snapshot_params: TargetSnapshotParams,
}

impl InProgressLoad {
    /// Create a new InProgressLoad object.
    pub(crate) fn new(new_pipeline_info: NewPipelineInfo) -> InProgressLoad {
        let url = new_pipeline_info.load_data.url.clone();
        InProgressLoad {
            pipeline_id: new_pipeline_info.new_pipeline_id,
            browsing_context_id: new_pipeline_info.browsing_context_id,
            webview_id: new_pipeline_info.webview_id,
            parent_info: new_pipeline_info.parent_info,
            opener: new_pipeline_info.opener,
            viewport_details: new_pipeline_info.viewport_details,
            activity: DocumentActivity::FullyActive,
            throttled: false,
            navigation_start: CrossProcessInstant::now(),
            canceller: Default::default(),
            load_data: new_pipeline_info.load_data,
            url_list: vec![url],
            user_content_manager_id: new_pipeline_info.user_content_manager_id,
            theme: new_pipeline_info.theme,
            target_snapshot_params: new_pipeline_info.target_snapshot_params,
        }
    }

    pub(crate) fn request_builder(&mut self) -> RequestBuilder {
        let client_origin = match self.load_data.load_origin {
            LoadOrigin::Script(ref initiator_origin) => initiator_origin.immutable().clone(),
            _ => ImmutableOrigin::new_opaque(),
        };

        let id = self.pipeline_id;
        let webview_id = self.webview_id;

        let insecure_requests_policy = self
            .load_data
            .inherited_insecure_requests_policy
            .unwrap_or(InsecureRequestsPolicy::DoNotUpgrade);

        let request_client = RequestClient {
            preloaded_resources: PreloadedResources::default(),
            policy_container: RequestPolicyContainer::PolicyContainer(
                self.load_data.policy_container.clone().unwrap_or_default(),
            ),
            origin: Origin::Origin(client_origin),
            is_nested_browsing_context: self.parent_info.is_some(),
            insecure_requests_policy,
        };

        let mut request_builder = RequestBuilder::new(
            Some(webview_id),
            UrlWithBlobClaim::from_url_without_having_claimed_blob(self.load_data.url.clone()),
            self.load_data.referrer.clone(),
        )
        .method(self.load_data.method.clone())
        .destination(self.load_data.destination)
        .mode(RequestMode::Navigate)
        .credentials_mode(CredentialsMode::Include)
        .use_url_credentials(true)
        .pipeline_id(Some(id))
        .referrer_policy(self.load_data.referrer_policy)
        .policy_container(self.load_data.policy_container.clone().unwrap_or_default())
        .insecure_requests_policy(insecure_requests_policy)
        .has_trustworthy_ancestor_origin(self.load_data.has_trustworthy_ancestor_origin)
        .headers(self.load_data.headers.clone())
        .body(self.load_data.data.clone())
        .redirect_mode(RedirectMode::Manual)
        .crash(self.load_data.crash.clone())
        .client(request_client)
        .url_list(self.url_list.clone());

        if !request_builder.headers.contains_key(header::ACCEPT) {
            request_builder
                .headers
                .insert(header::ACCEPT, DOCUMENT_ACCEPT_HEADER_VALUE);
        }
        set_default_accept_language(&mut request_builder.headers);

        request_builder
    }
}

/// <https://html.spec.whatwg.org/multipage/#determining-the-origin>
pub(crate) fn determine_the_origin(
    url: Option<&ServoUrl>,
    sandbox_flags: SandboxingFlagSet,
    source_origin: Option<MutableOrigin>,
) -> MutableOrigin {
    // Step 1. If sandboxFlags has its sandboxed origin browsing context flag set, then return a new opaque origin.
    let is_sandboxed =
        sandbox_flags.contains(SandboxingFlagSet::SANDBOXED_ORIGIN_BROWSING_CONTEXT_FLAG);
    if is_sandboxed {
        return MutableOrigin::new(ImmutableOrigin::new_opaque());
    }

    // Step 2. If url is null, then return a new opaque origin.
    let Some(url) = url else {
        return MutableOrigin::new(ImmutableOrigin::new_opaque());
    };

    // Step 3. If url is about:srcdoc, then:
    if url.as_str() == "about:srcdoc" {
        // Step 3.1 Assert: sourceOrigin is non-null.
        let source_origin =
            source_origin.expect("Can't have a null source origin for about:srcdoc");
        // Step 3.2 Return sourceOrigin
        return source_origin;
    }

    // Step 4. If url matches about:blank and sourceOrigin is non-null, then return sourceOrigin.
    if url.as_str() == "about:blank" {
        if let Some(source_origin) = source_origin {
            return source_origin;
        }
    }

    // Step 5. Return url's origin.
    MutableOrigin::new(url.origin())
}

/// <https://html.spec.whatwg.org/multipage/#navigate-fragid>
fn navigate_to_fragment(
    cx: &mut JSContext,
    window: &Window,
    url: &ServoUrl,
    history_handling: NavigationHistoryBehavior,
) {
    let doc = window.Document();
    // Step 1. Let navigation be navigable's active window's navigation API.
    // TODO
    // Step 2. Let destinationNavigationAPIState be navigable's active session history entry's navigation API state.
    // TODO
    // Step 3. If navigationAPIState is not null, then set destinationNavigationAPIState to navigationAPIState.
    // TODO

    // Step 4. Let continue be the result of firing a push/replace/reload navigate event
    // at navigation with navigationType set to historyHandling, isSameDocument set to true,
    // userInvolvement set to userInvolvement, sourceElement set to sourceElement,
    // destinationURL set to url, and navigationAPIState set to destinationNavigationAPIState.
    // TODO
    // Step 5. If continue is false, then return.
    // TODO

    // Step 6. Let historyEntry be a new session history entry, with
    // Step 7. Let entryToReplace be navigable's active session history entry if historyHandling is "replace", otherwise null.
    // Step 8. Let history be navigable's active document's history object.
    // Step 9. Let scriptHistoryIndex be history's index.
    // Step 10. Let scriptHistoryLength be history's length.
    // Step 11. If historyHandling is "push", then:
    // Step 13. Set navigable's active session history entry to historyEntry.
    window.send_to_constellation(ScriptToConstellationMessage::NavigatedToFragment(
        url.clone(),
        history_handling,
    ));
    // Step 12. Set navigable's active document's URL to url.
    let old_url = doc.url();
    doc.set_url(url.clone());
    // Step 14. Update document for history step application given navigable's active document,
    // historyEntry, true, scriptHistoryIndex, scriptHistoryLength, and historyHandling.
    doc.update_document_for_history_step_application(&old_url, url);
    // Step 15. Scroll to the fragment given navigable's active document.
    let Some(fragment) = url.fragment() else {
        unreachable!("Must always have a fragment");
    };
    doc.scroll_to_the_fragment(fragment, CanGc::from_cx(cx));
    // Step 16. Let traversable be navigable's traversable navigable.
    // TODO
    // Step 17. Append the following session history synchronous navigation steps involving navigable to traversable:
    // TODO
}

/// <https://html.spec.whatwg.org/multipage/#navigate>
pub(crate) fn navigate(
    cx: &mut JSContext,
    window: &Window,
    history_handling: NavigationHistoryBehavior,
    force_reload: bool,
    load_data: LoadData,
) {
    let doc = window.Document();

    // Step 3. Let initiatorOriginSnapshot be sourceDocument's origin.
    let initiator_origin_snapshot = &load_data.load_origin;

    // TODO: Important re security. See https://github.com/servo/servo/issues/23373
    // Step 5. check that the source browsing-context is "allowed to navigate" this window.

    // Step 4 and 5
    let pipeline_id = window.pipeline_id();
    let window_proxy = window.window_proxy();
    if let Some(active) = window_proxy.currently_active() {
        if pipeline_id == active && doc.is_prompting_or_unloading() {
            return;
        }
    }

    // Step 12. If historyHandling is "auto", then:
    let history_handling = if history_handling == NavigationHistoryBehavior::Auto {
        // Step 12.1. If url equals navigable's active document's URL, and
        // initiatorOriginSnapshot is same origin with targetNavigable's active document's
        // origin, then set historyHandling to "replace".
        //
        // Note: `targetNavigable` is not actually defined in the spec, "active document" is
        // assumed to be the correct reference based on WPT results
        if let LoadOrigin::Script(initiator_origin) = initiator_origin_snapshot {
            if load_data.url == doc.url() && initiator_origin.same_origin(&*doc.origin()) {
                NavigationHistoryBehavior::Replace
            } else {
                // Step 12.2. Otherwise, set historyHandling to "push".
                NavigationHistoryBehavior::Push
            }
        } else {
            // Step 12.2. Otherwise, set historyHandling to "push".
            NavigationHistoryBehavior::Push
        }
    } else {
        history_handling
    };

    // Step 13. If the navigation must be a replace given url and navigable's active
    // document, then set historyHandling to "replace".
    //
    // Inlines implementation of https://html.spec.whatwg.org/multipage/#the-navigation-must-be-a-replace
    let history_handling = if load_data.url.scheme() == "javascript" || doc.is_initial_about_blank()
    {
        NavigationHistoryBehavior::Replace
    } else {
        history_handling
    };

    // Step 14. If all of the following are true:
    // > documentResource is null;
    // > response is null;
    if !force_reload
        // > url equals navigable's active session history entry's URL with exclude fragments set to true; and
        && load_data.url.as_url()[..Position::AfterQuery] ==
            doc.url().as_url()[..Position::AfterQuery]
        // > url's fragment is non-null,
        && load_data.url.fragment().is_some()
    {
        // Step 14.1. Navigate to a fragment given navigable, url, historyHandling,
        // userInvolvement, sourceElement, navigationAPIState, and navigationId.
        let webdriver_sender = window.webdriver_load_status_sender();
        if let Some(ref sender) = webdriver_sender {
            let _ = sender.send(WebDriverLoadStatus::NavigationStart);
        }
        navigate_to_fragment(cx, window, &load_data.url, history_handling);
        // Step 14.2. Return.
        if let Some(sender) = webdriver_sender {
            let _ = sender.send(WebDriverLoadStatus::NavigationStop);
        }
        return;
    }

    // Step 15. If navigable's parent is non-null, then set navigable's is delaying load events to true.
    let window_proxy = window.window_proxy();
    if window_proxy.parent().is_some() {
        window_proxy.start_delaying_load_events_mode();
    }

    // Step 16. Let targetSnapshotParams be the result of snapshotting target
    // snapshot params given navigable.
    let target_snapshot_params = snapshot_target_snapshot_params(&window_proxy);

    // Step 17. Invoke WebDriver BiDi navigation started with navigable
    // and a new WebDriver BiDi navigation status whose id is navigationId,
    // status is "pending", and url is url.
    // TODO
    if let Some(sender) = window.webdriver_load_status_sender() {
        let _ = sender.send(WebDriverLoadStatus::NavigationStart);
    }

    // Step 18. If navigable's ongoing navigation is "traversal", then:
    // TODO
    // Step 19. Set the ongoing navigation for navigable to navigationId.
    // TODO

    // Step 20. If url's scheme is "javascript", then:
    if load_data.url.scheme() == "javascript" {
        // Step 20.1. Queue a global task on the navigation and traversal task source given
        // navigable's active window to navigate to a javascript: URL given navigable, url,
        // historyHandling, sourceSnapshotParams, initiatorOriginSnapshot, userInvolvement,
        // cspNavigationType, initialInsertion, and navigationId.
        let global = window.as_global_scope();
        let trusted_window = Trusted::new(window);
        let sender = global.script_to_constellation_chan().clone();
        let mut load_data = load_data;
        load_data.about_base_url = window.Document().about_base_url();
        let task = task!(navigate_javascript: move |cx| {
            // Important re security. See https://github.com/servo/servo/issues/23373
            let window = trusted_window.root();
            let global = window.as_global_scope();
            if ScriptThread::navigate_to_javascript_url(cx, global, global, &mut load_data, None, None) {
                sender
                    .send(ScriptToConstellationMessage::LoadUrl(load_data, history_handling, target_snapshot_params))
                    .unwrap();
            }
        });
        global
            .task_manager()
            .navigation_and_traversal_task_source()
            .queue(task);
        // Step 20.2. Return.
        return;
    }

    // Step 23. In parallel, run these steps:
    //
    // TODO: in parallel

    // Step 23.1. Let unloadPromptCanceled be the result of checking if unloading
    // is canceled for navigable's active document's inclusive descendant navigables.
    let unload_prompt_canceled = doc.check_if_unloading_is_cancelled(false, CanGc::from_cx(cx));
    // Step 23.2. If unloadPromptCanceled is not "continue",
    // or navigable's ongoing navigation is no longer navigationId:
    //
    // TODO: Check for ongoing navigation
    if !unload_prompt_canceled {
        // Step 23.2.1. Invoke WebDriver BiDi navigation failed with navigable
        // and a new WebDriver BiDi navigation status whose id is navigationId,
        // status is "canceled", and url is url.
        // TODO
        // Step 23.2.2. Abort these steps.
        return;
    }

    // Step 23.9. Attempt to populate the history entry's document for historyEntry,
    // given navigable, "navigate", sourceSnapshotParams, targetSnapshotParams,
    // userInvolvement, navigationId, navigationParams, cspNavigationType,
    // with allowPOST set to true and completionSteps set to the following step:
    window.send_to_constellation(ScriptToConstellationMessage::LoadUrl(
        load_data,
        history_handling,
        target_snapshot_params,
    ));
}

/// <https://html.spec.whatwg.org/multipage/#determining-the-creation-sandboxing-flags>
pub(crate) fn determine_creation_sandboxing_flags(
    browsing_context: Option<&WindowProxy>,
    element: Option<&Element>,
) -> SandboxingFlagSet {
    // To determine the creation sandboxing flags for a browsing context
    // browsing context, given null or an element embedder, return the union
    // of the flags that are present in the following sandboxing flag sets:
    match element {
        // If embedder is null, then: the flags set on browsing context's
        // popup sandboxing flag set.
        None => browsing_context
            .and_then(|browsing_context| browsing_context.document())
            .map(|document| document.active_sandboxing_flag_set())
            .unwrap_or(SandboxingFlagSet::empty()),
        Some(element) => {
            // If embedder is an element, then: the flags set on embedder's
            // iframe sandboxing flag set.
            // If embedder is an element, then: the flags set on embedder's
            // node document's active sandboxing flag set.
            element
                .downcast::<HTMLIFrameElement>()
                .map(|iframe| iframe.sandboxing_flag_set())
                .unwrap_or(SandboxingFlagSet::empty())
                .union(element.owner_document().active_sandboxing_flag_set())
        },
    }
}

/// <https://html.spec.whatwg.org/multipage/#determining-the-iframe-element-referrer-policy>
pub(crate) fn determine_iframe_element_referrer_policy(
    element: Option<&Element>,
) -> ReferrerPolicy {
    // Step 1. If embedder is an iframe element, then return embedder's referrerpolicy
    // attribute's state's corresponding keyword.
    element
        .and_then(|element| element.downcast::<HTMLIFrameElement>())
        .map(|iframe| {
            let token = iframe.ReferrerPolicy();
            ReferrerPolicy::from(&*token.str())
        })
        // Step 2. Return the empty string.
        .unwrap_or(ReferrerPolicy::EmptyString)
}

/// <https://html.spec.whatwg.org/multipage/#snapshotting-target-snapshot-params>
pub(crate) fn snapshot_target_snapshot_params(navigable: &WindowProxy) -> TargetSnapshotParams {
    // TODO(jdm): This doesn't work for cross-origin parent frames.
    let container = navigable.frame_element();
    // the result of determining the creation sandboxing flags given targetNavigable's
    // active browsing context and targetNavigable's container
    let sandboxing_flags = determine_creation_sandboxing_flags(Some(navigable), container);
    // the result of determining the iframe element referrer policy given
    // targetNavigable's container
    let iframe_element_referrer_policy = determine_iframe_element_referrer_policy(container);
    TargetSnapshotParams {
        sandboxing_flags,
        iframe_element_referrer_policy,
    }
}
