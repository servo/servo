/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::Cell;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use dom_struct::dom_struct;
use embedder_traits::{
    AllowOrDeny, EmbedderMsg, GeolocationError, GeolocationRequestOptions, GeolocationResult,
    GeolocationWatchId,
};
use js::context::JSContext;
use rustc_hash::{FxHashMap, FxHashSet};
use script_bindings::callback::{ExceptionHandling, OwnerWindow, RootedCallback, TracedCallback};
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::DocumentBinding::{
    DocumentMethods, DocumentVisibilityState,
};
use script_bindings::codegen::GenericBindings::GeolocationBinding::Geolocation_Binding::GeolocationMethods;
use script_bindings::codegen::GenericBindings::GeolocationBinding::{
    PositionCallback, PositionErrorCallback, PositionOptions,
};
use script_bindings::codegen::GenericBindings::PermissionStatusBinding::{
    PermissionName, PermissionState,
};
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::domstring::DOMString;
use script_bindings::reflector::{Reflector, reflect_dom_object};
use script_bindings::root::DomRoot;
use servo_base::generic_channel::GenericCallback;

use crate::dom::bindings::codegen::DomTypeHolder::DomTypeHolder;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::MutNullableDom;
use crate::dom::geolocationposition::GeolocationPosition;
use crate::dom::geolocationpositionerror::GeolocationPositionError;
use crate::dom::globalscope::GlobalScope;
use crate::dom::permissions::{
    AsyncPermissionRequest, descriptor_permission_state, record_permission_request_result,
    request_permission_to_use_async,
};
use crate::event_loop::timers::{OneshotTimerCallback, OneshotTimerHandle};

/// Identifies one in-flight *request a position* algorithm.
type RequestId = u32;

/// The `PositionOptions` a request was started with.
#[derive(Clone, JSTraceable, MallocSizeOf)]
struct RequestOptions {
    /// <https://www.w3.org/TR/geolocation/#dom-positionoptions-enablehighaccuracy>
    enable_high_accuracy: bool,
    /// <https://www.w3.org/TR/geolocation/#dom-positionoptions-timeout>
    timeout: u32,
    /// <https://www.w3.org/TR/geolocation/#dom-positionoptions-maximumage>
    maximum_age: u32,
}

impl From<&PositionOptions> for RequestOptions {
    fn from(options: &PositionOptions) -> Self {
        Self {
            enable_high_accuracy: options.enableHighAccuracy,
            timeout: options.timeout,
            maximum_age: options.maximumAge,
        }
    }
}

impl From<RequestOptions> for GeolocationRequestOptions {
    fn from(val: RequestOptions) -> Self {
        GeolocationRequestOptions {
            high_accuracy: val.enable_high_accuracy,
        }
    }
}

/// The state of a single in-flight call to `getCurrentPosition()` or `watchPosition()`.
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct PositionRequest {
    success_callback: TracedCallback<PositionCallback<DomTypeHolder>>,
    error_callback: Option<TracedCallback<PositionErrorCallback<DomTypeHolder>>>,
    options: RequestOptions,
    /// The `watchId` returned by `watchPosition()`, or `None` for `getCurrentPosition()`.
    watch_id: Option<u32>,
    /// The timeout armed for the acquisition currently under way, if any.
    timeout_handle: Cell<Option<OneshotTimerHandle>>,
    /// Set once the current acquisition has produced a result, so that a reply racing the timeout
    /// is ignored.
    settled: Cell<bool>,
    /// Whether the embedder has been asked to keep this watch updated.
    watching: Cell<bool>,
    /// Set when the request has failed for good, so that a watch is forgotten once its error
    /// callback has run rather than waiting for updates that will never come.
    terminated: Cell<bool>,
}

impl PositionRequest {
    fn is_watch(&self) -> bool {
        self.watch_id.is_some()
    }
}

/// What a queued geolocation task will report to script.
enum Completion {
    Success(Trusted<GeolocationPosition>),
    Failure(PositionError),
}

/// The reason a request failed, resolved into a `GeolocationPositionError` once the
/// geolocation task runs.
enum PositionError {
    PermissionDenied(&'static str),
    PositionUnavailable(&'static str),
    Timeout,
    /// An error reported by the embedding layer's geolocation backend.
    Embedder(GeolocationError),
}

impl PositionError {
    fn create(
        &self,
        cx: &mut JSContext,
        global: &GlobalScope,
    ) -> DomRoot<GeolocationPositionError> {
        match self {
            Self::PermissionDenied(message) => {
                GeolocationPositionError::permission_denied(cx, global, DOMString::from(*message))
            },
            Self::PositionUnavailable(message) => GeolocationPositionError::position_unavailable(
                cx,
                global,
                DOMString::from(*message),
            ),
            Self::Timeout => GeolocationPositionError::timeout(
                cx,
                global,
                DOMString::from_static("Acquiring the position timed out"),
            ),
            Self::Embedder(error) => {
                GeolocationPositionError::from_embedder_error(cx, global, error)
            },
        }
    }
}

/// <https://www.w3.org/TR/geolocation/#dfn-epochtimestamp>
fn now_as_epoch_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

#[dom_struct]
pub struct Geolocation {
    reflector_: Reflector,
    /// <https://www.w3.org/TR/geolocation/#dfn-watchids>
    watch_ids: DomRefCell<FxHashSet<u32>>,
    next_watch_id: Cell<u32>,
    /// In-flight requests, keyed by the internal identifier.
    requests: DomRefCell<FxHashMap<RequestId, PositionRequest>>,
    next_request_id: Cell<RequestId>,
    /// Requests parked by step 5 of "request a position" because the document is hidden. They
    /// resume when it becomes visible again.
    deferred_until_visible: DomRefCell<Vec<RequestId>>,
    /// <https://www.w3.org/TR/geolocation/#dfn-cachedposition>
    cached_position: MutNullableDom<GeolocationPosition>,
}

impl Geolocation {
    fn new_inherited() -> Self {
        Geolocation {
            reflector_: Reflector::new(),
            watch_ids: Default::default(),
            next_watch_id: Cell::new(1),
            requests: Default::default(),
            next_request_id: Cell::new(1),
            deferred_until_visible: Default::default(),
            cached_position: Default::default(),
        }
    }

    pub(crate) fn new(cx: &mut JSContext, global: &GlobalScope) -> DomRoot<Self> {
        reflect_dom_object(cx, Box::new(Self::new_inherited()), global)
    }

    fn register_request(
        &self,
        success_callback: RootedCallback<PositionCallback<DomTypeHolder>>,
        error_callback: Option<RootedCallback<PositionErrorCallback<DomTypeHolder>>>,
        options: &PositionOptions,
        watch_id: Option<u32>,
    ) -> RequestId {
        let request_id = self.next_request_id.get();
        self.next_request_id.set(request_id.wrapping_add(1));
        self.requests.borrow_mut().insert(
            request_id,
            PositionRequest {
                success_callback: success_callback.to_traced(),
                error_callback: error_callback.map(|callback| callback.to_traced()),
                options: options.into(),
                watch_id,
                timeout_handle: Cell::new(None),
                settled: Cell::new(false),
                watching: Cell::new(false),
                terminated: Cell::new(false),
            },
        );
        request_id
    }

    /// Forget a request, disarming its timeout and ending any embedder subscription it owns.
    fn discard_request(&self, request_id: RequestId) {
        let Some((timeout_handle, watch_id, watching)) =
            self.requests.borrow().get(&request_id).map(|request| {
                (
                    request.timeout_handle.take(),
                    request.watch_id,
                    request.watching.get(),
                )
            })
        else {
            return;
        };
        self.requests.borrow_mut().remove(&request_id);

        if let Some(handle) = timeout_handle {
            self.global().unschedule_callback(handle);
        }
        if let (Some(watch_id), true) = (watch_id, watching) {
            self.stop_embedder_watch(watch_id);
        }
    }

    fn disarm_timeout(&self, request_id: RequestId) {
        let handle = self
            .requests
            .borrow()
            .get(&request_id)
            .and_then(|request| request.timeout_handle.take());
        if let Some(handle) = handle {
            self.global().unschedule_callback(handle);
        }
    }

    fn embedder_watch_id(&self, watch_id: u32) -> Option<GeolocationWatchId> {
        let global = self.global();
        Some(GeolocationWatchId {
            webview_id: global.webview_id()?,
            pipeline_id: global.pipeline_id(),
            index: watch_id,
        })
    }

    fn stop_embedder_watch(&self, watch_id: u32) {
        let Some(id) = self.embedder_watch_id(watch_id) else {
            return;
        };
        self.global()
            .send_to_embedder(EmbedderMsg::StopGeolocationWatch(id));
    }

    /// Stop every ongoing watch. Called when the `Window` is torn down, so that the embedder does
    /// not keep a location subscription alive for a pipeline that no longer exists.
    pub(crate) fn stop_all_watches(&self) {
        self.deferred_until_visible.borrow_mut().clear();
        self.watch_ids.borrow_mut().clear();

        let outstanding: Vec<_> = self
            .requests
            .borrow()
            .values()
            .map(|request| {
                (
                    request.timeout_handle.take(),
                    request.watch_id,
                    request.watching.get(),
                )
            })
            .collect();
        self.requests.borrow_mut().clear();

        for (timeout_handle, watch_id, watching) in outstanding {
            if let Some(handle) = timeout_handle {
                self.global().unschedule_callback(handle);
            }
            if let (Some(watch_id), true) = (watch_id, watching) {
                self.stop_embedder_watch(watch_id);
            }
        }
    }

    /// Whether the requesting Document may currently be given a position. Per step 7.5.2 of
    /// "request a position", updates are exclusively for fully-active visible documents;
    /// otherwise they are silently dropped.
    ///
    /// <https://www.w3.org/TR/geolocation/#dfn-request-a-position>
    fn document_can_receive_positions(&self) -> bool {
        let document = self.global().as_window().Document();
        document.is_fully_active() && document.VisibilityState() == DocumentVisibilityState::Visible
    }

    /// Queue a task on the geolocation task source that reports `completion` to the callbacks of
    /// `request_id`. Once reported, a `getCurrentPosition()` request is finished and forgotten,
    /// while a watch stays registered to receive further updates.
    ///
    /// <https://www.w3.org/TR/geolocation/#dfn-call-back-with-error>
    fn queue_completion(&self, request_id: RequestId, completion: Completion) {
        let this = Trusted::new(self);
        self.global()
            .task_manager()
            .geolocation_task_source()
            .queue(task!(geolocation_completion: move |cx| {
                this.root().report_completion(cx, request_id, completion);
            }));
    }

    /// Invoke the callbacks of `request_id` with the result of an acquisition.
    fn report_completion(&self, cx: &mut JSContext, request_id: RequestId, completion: Completion) {
        let Some(keep_listening) = self
            .requests
            .borrow()
            .get(&request_id)
            .map(|request| request.is_watch() && !request.terminated.get())
        else {
            return;
        };

        match completion {
            Completion::Success(position) => {
                rooted!(&in(cx) let callback = self
                    .requests
                    .borrow()
                    .get(&request_id)
                    .map(|request| request.success_callback.clone()));
                if let Some(ref callback) = *callback {
                    let position = position.root();
                    let _ = callback.Call_(cx, self, &position, ExceptionHandling::Report);
                }
            },
            Completion::Failure(error) => {
                // Step 1 of `call back with error`: if callback is null, return.
                rooted!(&in(cx) let callback = self
                    .requests
                    .borrow()
                    .get(&request_id)
                    .and_then(|request| request.error_callback.clone()));
                if let Some(ref callback) = *callback {
                    let position_error = error.create(cx, &self.global());
                    let _ = callback.Call_(cx, self, &position_error, ExceptionHandling::Report);
                }
            },
        }

        if !keep_listening {
            self.requests.borrow_mut().remove(&request_id);
        }
    }

    /// Report an error and terminate the request, whether or not it is a watch.
    fn terminate_with_error(&self, request_id: RequestId, error: PositionError) {
        let Some((watch_id, was_watching)) =
            self.requests.borrow().get(&request_id).map(|request| {
                request.settled.set(true);
                request.terminated.set(true);
                (request.watch_id, request.watching.replace(false))
            })
        else {
            return;
        };
        self.disarm_timeout(request_id);
        self.remove_watch_id(watch_id);
        if let (Some(watch_id), true) = (watch_id, was_watching) {
            self.stop_embedder_watch(watch_id);
        }
        self.queue_completion(request_id, Completion::Failure(error));
    }

    /// Finish the acquisition currently under way and report `completion`. A watch stays
    /// registered, and the next update from the embedder begins a fresh acquisition.
    fn complete_acquisition(&self, request_id: RequestId, completion: Completion) {
        self.queue_completion(request_id, completion);
        if let Some(request) = self.requests.borrow().get(&request_id) &&
            request.is_watch()
        {
            request.settled.set(false);
        }
    }

    /// <https://www.w3.org/TR/geolocation/#dfn-request-a-position>
    fn request_position(
        &self,
        success_callback: RootedCallback<PositionCallback<DomTypeHolder>>,
        error_callback: Option<RootedCallback<PositionErrorCallback<DomTypeHolder>>>,
        options: &PositionOptions,
        watch_id: Option<u32>,
    ) {
        let request_id = self.register_request(success_callback, error_callback, options, watch_id);

        // Step 1. Let watchIDs be geolocation's [[watchIDs]].
        // Step 2. Let document be the geolocation's relevant global object's associated Document.
        let document = self.global().as_window().Document();

        // Step 3. If document is not allowed to use the "geolocation" feature:
        if !document.allowed_to_use_feature(PermissionName::Geolocation) {
            // Step 3.1, 3.2 and 3.3.
            self.terminate_with_error(
                request_id,
                PositionError::PermissionDenied("User denied Geolocation"),
            );
            return;
        }

        // Step 4. If geolocation's environment settings object is a non-secure context:
        if !self.global().is_secure_context() {
            // Step 4.1, 4.2 and 4.3.
            self.terminate_with_error(
                request_id,
                PositionError::PermissionDenied("Insecure context for Geolocation"),
            );
            return;
        }

        // Step 5. If document's visibility state is "hidden", wait for the page visibility change
        // steps to run.
        if document.VisibilityState() == DocumentVisibilityState::Hidden {
            self.deferred_until_visible.borrow_mut().push(request_id);
            return;
        }

        // Step 6 and 7.
        self.obtain_permission_then_acquire(request_id);
    }

    /// Step 6 and 7 of <https://www.w3.org/TR/geolocation/#dfn-request-a-position>: obtain
    /// permission for the "geolocation" descriptor, then acquire a position.
    fn obtain_permission_then_acquire(&self, request_id: RequestId) {
        if !self.requests.borrow().contains_key(&request_id) {
            // `clearWatch()` reached the request while it was parked waiting for the document to
            // become visible. Asking the user for permission now would be a prompt for nothing.
            return;
        }

        // Step 6. Let descriptor be a new PermissionDescriptor whose name is "geolocation".
        // Step 7. In parallel, set permission to request permission to use descriptor.
        let global = self.global();
        let this = Trusted::new(self);
        let task_source = global
            .task_manager()
            .geolocation_task_source()
            .to_sendable();
        let callback = GenericCallback::new(move |response: Result<AllowOrDeny, _>| {
            let this = this.clone();
            task_source.queue(task!(geolocation_permission_response: move |_cx| {
                let this = this.root();
                let state = match response {
                    Ok(response) => record_permission_request_result(
                        PermissionName::Geolocation,
                        &this.global(),
                        response,
                    ),
                    Err(_) => PermissionState::Denied,
                };
                this.handle_permission_state(request_id, state);
            }));
        })
        .expect("Could not create callback in script.");

        match request_permission_to_use_async(PermissionName::Geolocation, &global, callback) {
            AsyncPermissionRequest::Settled(state) => {
                self.handle_permission_state(request_id, state)
            },
            AsyncPermissionRequest::Prompting => {},
        }
    }

    fn handle_permission_state(&self, request_id: RequestId, state: PermissionState) {
        if state == PermissionState::Granted {
            // Step 7.3. Wait to acquire a position.
            self.acquire_position(request_id);
        } else {
            // Step 7.2. If permission is "denied", remove watchId from watchIDs and call back
            // with error passing errorCallback and PERMISSION_DENIED.
            self.terminate_with_error(
                request_id,
                PositionError::PermissionDenied("User denied Geolocation"),
            );
        }
    }

    /// <https://www.w3.org/TR/geolocation/#dfn-acquire-a-position>
    fn acquire_position(&self, request_id: RequestId) {
        let Some((options, watch_id)) = self
            .requests
            .borrow()
            .get(&request_id)
            .map(|request| (request.options.clone(), request.watch_id))
        else {
            return;
        };

        // Step 1. If watchId was passed and [[watchIDs]] does not contain watchId, terminate.
        if let Some(watch_id) = watch_id &&
            !self.watch_ids.borrow().contains(&watch_id)
        {
            self.discard_request(request_id);
            return;
        }

        if let Some(request) = self.requests.borrow().get(&request_id) {
            request.settled.set(false);
        }

        // Step 5.1 and 5.2. Re-check the permission state now that acquisition is starting; it
        // may have been revoked while the document was hidden.
        if descriptor_permission_state(PermissionName::Geolocation, Some(&self.global())) ==
            PermissionState::Denied
        {
            self.terminate_with_error(
                request_id,
                PositionError::PermissionDenied("User denied Geolocation"),
            );
            return;
        }

        // Step 2. Let acquisitionTime be a new EpochTimeStamp that represents now.
        let acquisition_time = now_as_epoch_timestamp();

        // Step 5.3.2 and 5.3.3. Reuse the cached position if it is fresh enough and was acquired
        // in the same accuracy mode.
        if options.maximum_age > 0 &&
            let Some(cached_position) = self.cached_position.get()
        {
            let cache_time = acquisition_time.saturating_sub(options.maximum_age as u64);
            if cached_position.timestamp() > cache_time &&
                cached_position.is_high_accuracy() == options.enable_high_accuracy
            {
                self.settle(request_id);
                self.complete_acquisition(
                    request_id,
                    Completion::Success(Trusted::new(&*cached_position)),
                );
                return;
            }
        }

        // Step 3 and 5. The timeout bounds the acquisition that follows, so a zero timeout leaves
        // no window at all. Fail immediately rather than troubling the embedder.
        if options.timeout == 0 {
            self.settle(request_id);
            self.complete_acquisition(request_id, Completion::Failure(PositionError::Timeout));
            return;
        }

        // A watch arms its timeout only for the initial fix. Later updates arrive as the device
        // moves, and re-arming would report a timeout every time it stands still.
        self.arm_timeout(request_id, options.timeout);

        // Step 5.3.4. Try to acquire position data from the underlying system.
        let Some(embedder_msg) = self.build_acquisition_message(request_id, watch_id, options)
        else {
            self.settle(request_id);
            self.complete_acquisition(
                request_id,
                Completion::Failure(PositionError::PositionUnavailable(
                    "No geolocation backend is available.",
                )),
            );
            return;
        };
        if watch_id.is_some() &&
            let Some(request) = self.requests.borrow().get(&request_id)
        {
            request.watching.set(true);
        }
        self.global().send_to_embedder(embedder_msg);
    }

    /// Ask the embedding layer for a position, either once or as an ongoing subscription.
    fn build_acquisition_message(
        &self,
        request_id: RequestId,
        watch_id: Option<u32>,
        options: RequestOptions,
    ) -> Option<EmbedderMsg> {
        let global = self.global();
        let webview_id = global.webview_id()?;
        let this = Trusted::new(self);
        let task_source = global
            .task_manager()
            .geolocation_task_source()
            .to_sendable();
        let callback = GenericCallback::new(move |result: Result<GeolocationResult, _>| {
            let this = this.clone();
            let result = result.unwrap_or_else(|error| {
                warn!("Failed to receive a position from the embedder ({error:?}).");
                Err(GeolocationError::position_unavailable(
                    "Failed to communicate with the geolocation backend.",
                ))
            });
            task_source.queue(task!(geolocation_position_acquired: move |cx| {
                this.root().handle_acquisition_result(cx, request_id, result);
            }));
        })
        .expect("Could not create callback in script.");

        Some(match watch_id {
            Some(watch_id) => EmbedderMsg::StartGeolocationWatch(
                self.embedder_watch_id(watch_id)?,
                options.into(),
                callback,
            ),
            None => EmbedderMsg::RequestGeolocationPosition(webview_id, options.into(), callback),
        })
    }

    /// Handle one reply from the embedding layer. A one-shot request is finished by the first
    /// reply; for a watch, each further reply is a "significant change of geographic position".
    fn handle_acquisition_result(
        &self,
        cx: &mut JSContext,
        request_id: RequestId,
        result: GeolocationResult,
    ) {
        let Some((options, watch_id, is_watch)) =
            self.requests.borrow().get(&request_id).map(|request| {
                (
                    request.options.clone(),
                    request.watch_id,
                    request.is_watch(),
                )
            })
        else {
            return;
        };

        // A watch that was cleared while a reply was in flight produces nothing. A watch that
        // was terminated by an error is left alone: its completion task will remove it once the
        // error callback has run.
        if let Some(watch_id) = watch_id &&
            !self.watch_ids.borrow().contains(&watch_id)
        {
            let terminated = self
                .requests
                .borrow()
                .get(&request_id)
                .is_some_and(|request| request.terminated.get());
            if !terminated {
                self.discard_request(request_id);
            }
            return;
        }

        // Step 7.5.2 of `request a position`: drop updates for documents that are not fully
        // active and visible.
        if is_watch && !self.document_can_receive_positions() {
            return;
        }

        // The timeout already fired, or an earlier reply already settled this acquisition.
        if !self.settle(request_id) {
            return;
        }

        let completion = match result {
            Ok(data) => {
                // Step 5.3.6. Set position to a new GeolocationPosition.
                match GeolocationPosition::from_position_data(
                    cx,
                    &self.global(),
                    &data,
                    now_as_epoch_timestamp(),
                    options.enable_high_accuracy,
                ) {
                    Some(position) => {
                        // Step 5.3.6.3. Set [[cachedPosition]] to position.
                        self.cached_position.set(Some(&position));
                        Completion::Success(Trusted::new(&*position))
                    },
                    None => Completion::Failure(PositionError::PositionUnavailable(
                        "The geolocation backend reported an unrepresentable position.",
                    )),
                }
            },
            Err(error) => Completion::Failure(PositionError::Embedder(error)),
        };
        self.complete_acquisition(request_id, completion);
    }

    /// Mark the current acquisition as finished and disarm its timeout. Returns `false` if it was
    /// already finished, in which case the caller should report nothing.
    fn settle(&self, request_id: RequestId) -> bool {
        let settled = self
            .requests
            .borrow()
            .get(&request_id)
            .map(|request| request.settled.replace(true));
        match settled {
            Some(false) => {
                self.disarm_timeout(request_id);
                true
            },
            _ => false,
        }
    }

    fn arm_timeout(&self, request_id: RequestId, timeout: u32) {
        let callback = OneshotTimerCallback::GeolocationTimeout(GeolocationTimeoutCallback {
            geolocation: Trusted::new(self),
            request_id,
        });
        let handle = self
            .global()
            .schedule_callback(callback, Duration::from_millis(timeout as u64));
        if let Some(request) = self.requests.borrow().get(&request_id) {
            request.timeout_handle.set(Some(handle));
        }
    }

    fn handle_timeout(&self, request_id: RequestId) {
        // The handle is consumed by firing, so forget it before settling.
        if let Some(request) = self.requests.borrow().get(&request_id) {
            request.timeout_handle.set(None);
        }
        if self.settle(request_id) {
            self.complete_acquisition(request_id, Completion::Failure(PositionError::Timeout));
        }
    }

    fn remove_watch_id(&self, watch_id: Option<u32>) {
        if let Some(watch_id) = watch_id {
            self.watch_ids.borrow_mut().remove(&watch_id);
        }
    }

    /// The geolocation part of the page visibility change steps: resume the requests parked by
    /// step 5 of "request a position".
    ///
    /// <https://www.w3.org/TR/geolocation/#dfn-request-a-position>
    pub(crate) fn handle_visibility_change(&self, visibility_state: DocumentVisibilityState) {
        if visibility_state != DocumentVisibilityState::Visible {
            return;
        }
        let deferred = std::mem::take(&mut *self.deferred_until_visible.borrow_mut());
        for request_id in deferred {
            self.obtain_permission_then_acquire(request_id);
        }
    }
}

impl GeolocationMethods<DomTypeHolder> for Geolocation {
    /// <https://www.w3.org/TR/geolocation/#dom-geolocation-getcurrentposition>
    fn GetCurrentPosition(
        &self,
        success_callback: RootedCallback<PositionCallback<DomTypeHolder>>,
        error_callback: Option<RootedCallback<PositionErrorCallback<DomTypeHolder>>>,
        options: &PositionOptions,
    ) {
        // Step 1. If this's relevant global object's associated Document is not fully active:
        if !self.global().as_window().Document().is_fully_active() {
            // Step 1.1 Call back with error errorCallback and POSITION_UNAVAILABLE.
            let request_id = self.register_request(success_callback, error_callback, options, None);
            self.queue_completion(
                request_id,
                Completion::Failure(PositionError::PositionUnavailable(
                    "Document is not fully active",
                )),
            );
            // Step 1.2 Terminate this algorithm.
            return;
        }
        // Step 2. Request a position passing this, successCallback, errorCallback, and options.
        self.request_position(success_callback, error_callback, options, None)
    }

    /// <https://www.w3.org/TR/geolocation/#watchposition-method>
    fn WatchPosition(
        &self,
        success_callback: RootedCallback<PositionCallback<DomTypeHolder>>,
        error_callback: Option<RootedCallback<PositionErrorCallback<DomTypeHolder>>>,
        options: &PositionOptions,
    ) -> i32 {
        // Step 1. If this's relevant global object's associated Document is not fully active:
        if !self.global().as_window().Document().is_fully_active() {
            // Step 1.1 Call back with error errorCallback and POSITION_UNAVAILABLE.
            let request_id = self.register_request(success_callback, error_callback, options, None);
            self.queue_completion(
                request_id,
                Completion::Failure(PositionError::PositionUnavailable(
                    "Document is not fully active",
                )),
            );
            // Step 1.2 Return 0.
            return 0;
        }
        // Step 2. Let watchId be an implementation-defined unsigned long that is greater than zero.
        let watch_id = self.next_watch_id.get();
        self.next_watch_id.set(watch_id + 1);
        // Step 3. Append watchId to this's [[watchIDs]].
        self.watch_ids.borrow_mut().insert(watch_id);
        // Step 4. Request a position passing this, successCallback, errorCallback, options, and
        // watchId.
        self.request_position(success_callback, error_callback, options, Some(watch_id));
        // Step 5. Return watchId.
        watch_id as i32
    }

    /// <https://www.w3.org/TR/geolocation/#clearwatch-method>
    fn ClearWatch(&self, watch_id: i32) {
        let Ok(watch_id) = u32::try_from(watch_id) else {
            return;
        };
        // Remove watchId from this's [[watchIDs]].
        self.remove_watch_id(Some(watch_id));

        let request_id = self
            .requests
            .borrow()
            .iter()
            .find(|(_, request)| request.watch_id == Some(watch_id))
            .map(|(request_id, _)| *request_id);
        if let Some(request_id) = request_id {
            self.discard_request(request_id);
        }
    }
}

impl OwnerWindow<crate::DomTypeHolder> for Geolocation {}

/// Fires when `PositionOptions.timeout` elapses before a position has been acquired.
#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct GeolocationTimeoutCallback {
    #[ignore_malloc_size_of = "Because it is non-owning"]
    geolocation: Trusted<Geolocation>,
    request_id: RequestId,
}

impl GeolocationTimeoutCallback {
    pub(crate) fn invoke(self) {
        self.geolocation.root().handle_timeout(self.request_id);
    }
}
