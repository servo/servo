use dom_struct::dom_struct;
use js::context::JSContext;
use js::jsval::UndefinedValue;
use script_bindings::callback::RootedCallback;
use script_bindings::codegen::GenericBindings::AbortSignalBinding::AbortSignalMethods;
use script_bindings::codegen::GenericBindings::WebLockBinding::{
    LockGrantedCallback, LockManagerMethods, LockMode, LockOptions,
};
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use script_bindings::root::{Dom, DomRoot, Root};
use script_bindings::str::DOMString;
use servo_base::generic_channel::{GenericCallback, GenericSend, SendResult};
use servo_url::ImmutableOrigin;
use storage_traits::weblocks::{
    LockInfoMsg, LockManagerSnapshotMsg, LockModeMsg, LockRequest, LockRequestId, WebLocksThreadMsg,
};

use crate::conversions::Convert;
use crate::dom::abortsignal::AbortAlgorithm;
use crate::dom::bindings::codegen::Bindings::WebLockBinding::{LockInfo, LockManagerSnapshot};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::types::{AbortSignal, DOMException};
use crate::dom::{GlobalScope, Promise, RootedPromise};
use crate::routed_promise::{RoutedPromiseListener, callback_promise};

#[dom_struct]
pub(crate) struct LockManager {
    reflector_: Reflector,
}

impl LockManager {
    fn new_inherited() -> LockManager {
        LockManager {
            reflector_: Reflector::new(),
        }
    }

    pub(crate) fn new(cx: &mut JSContext, global: &GlobalScope) -> DomRoot<LockManager> {
        reflect_dom_object_with_cx(Box::new(LockManager::new_inherited()), global, cx)
    }

    fn send_storage_msg(&self, msg: WebLocksThreadMsg) -> SendResult {
        GenericSend::send(self.global().storage_threads(), msg)
    }

    fn get_immutable_origin(&self) -> ImmutableOrigin {
        self.global().origin().immutable().clone()
    }

    /// <https://www.w3.org/TR/web-locks/#dom-lockmanager-request>
    fn request(
        &self,
        cx: &mut JSContext,
        name: DOMString,
        options: Option<&LockOptions<crate::DomTypeHolder>>,
        callback: RootedCallback<LockGrantedCallback<crate::DomTypeHolder>>,
    ) -> RootedPromise {
        // Step 1. If options was not passed, default LockOptions.
        let default_options = LockOptions::default();
        let options = options.unwrap_or(&default_options);

        // Step 2. Let environment be this’s relevant settings object.
        let global = self.global();

        let mut reject_not_supported = |message: &str| {
            Promise::new_rejected_rooted(
                cx,
                &global,
                DOMException::new_inherited(message.into(), "NotSupportedError".into()),
            )
        };

        // Step 3. If associated Document not fully active, reject with ...
        if !global.as_window().Document().is_fully_active() {
            return Promise::new_rejected_rooted(
                cx,
                &global,
                DOMException::new_inherited(
                    "associated Document is not fully active".into(),
                    "InvalidStateError".into(),
                ),
            );
        }

        // Step 4. skip, manager is on storage threads

        // Step 5-9. validate arguments
        if name.starts_with('-') {
            return reject_not_supported("name starts with U+002D HYPHEN-MINUS(-)");
        }
        if options.steal && options.ifAvailable {
            if options.ifAvailable {
                return reject_not_supported("both steal and ifAvailable are true");
            }
            if options.mode != LockMode::Exclusive {
                return reject_not_supported("steal is true but mode is not exclusive");
            }
        }
        if let Some(signal) = &options.signal {
            if options.steal || options.ifAvailable {
                return reject_not_supported(
                    "signal exists but either steal or ifAvailable is true",
                );
            }
            if signal.aborted() {
                rooted!(&in(cx) let mut reason = UndefinedValue());
                signal.Reason(reason.handle_mut());
                return Promise::new_rejected_rooted(cx, &global, *reason);
            }
        }

        // Step 10. Let promise be a new promise.
        let promise = Promise::new_rooted(cx, &global);
        // Step 11. Request a lock with arguments ...
        self.request_lock(
            cx,
            &promise,
            // TODO: environment id is not implemented in servo
            String::new(),
            callback,
            name,
            options.mode,
            options.ifAvailable,
            options.steal,
            &options.signal,
        );
        // Step 12. Return promise
        promise
    }

    /// <https://www.w3.org/TR/web-locks/#request-a-lock>
    #[expect(clippy::too_many_arguments)]
    fn request_lock(
        &self,
        cx: &mut JSContext,
        promise: &RootedPromise,
        client_id: String,
        callback: RootedCallback<LockGrantedCallback<crate::DomTypeHolder>>,
        name: DOMString,
        mode: LockMode,
        if_available: bool,
        steal: bool,
        signal: &Option<Root<Dom<AbortSignal>>>,
    ) {
        let origin = self.get_immutable_origin();

        // <https://www.w3.org/TR/web-locks/#process-the-lock-request-queue> Step 14 inner steps
        let held_callback = GenericCallback::new(|lock| {
            let lock = lock.unwrap();
        })
        .unwrap();

        // TODO: release seem should only later be sent to storage thread, otherwise the callback
        // cannot capture the waiting_promise.
        // NO: the release_promise can be rejected even without lock
        let released_callback = GenericCallback::new(|_err| {}).unwrap();

        // Step 1. Let request be a new lock request with ...
        let id = LockRequestId::next();
        let request = LockRequest {
            id,
            client_id,
            name: name.into(),
            mode: mode.convert(),
            held_callback,
            released_callback,
            if_available,
            steal,
        };

        // Step 2. If signal is present, add the abort algorithm to signal
        if let Some(signal) = signal {
            signal.add(&AbortAlgorithm::AbortLockRequest(
                id,
                origin.clone(),
                request.name.clone(),
                promise.clone(),
            ));
        }

        // Step 3. Enqueue the following steps to local task queue.
        // The inner steps (3.1 to 3.6) continues on WebLocksThread.
        if let Err(e) = self.send_storage_msg(WebLocksThreadMsg::Request(request, origin)) {
            warn!("Request failed with {e}");
        }

        // Step 4. skip return request, unused in spec
    }
}

impl LockManagerMethods<crate::DomTypeHolder> for LockManager {
    fn Request(
        &self,
        cx: &mut JSContext,
        name: DOMString,
        callback: RootedCallback<LockGrantedCallback<crate::DomTypeHolder>>,
    ) -> RootedPromise {
        self.request(cx, name, None, callback)
    }

    fn Request_(
        &self,
        cx: &mut JSContext,
        name: DOMString,
        options: &LockOptions<crate::DomTypeHolder>,
        callback: RootedCallback<LockGrantedCallback<crate::DomTypeHolder>>,
    ) -> RootedPromise {
        self.request(cx, name, Some(options), callback)
    }

    fn Query(&self, cx: &mut JSContext) -> RootedPromise {
        // Step 1. Let environment be this’s relevant settings object.
        let global = self.global();

        // Step 2. If associated Document is not fully active, reject with ...
        if !global.as_window().Document().is_fully_active() {
            return Promise::new_rejected_rooted(
                cx,
                &global,
                DOMException::new_inherited(
                    "associated Document is not fully active".into(),
                    "InvalidStateError".into(),
                ),
            );
        }

        // Step 3. skip, manager is on WebLocksThread

        // Step 4. Let promise be a new promise.
        let promise = Promise::new_rooted(cx, &global);

        // Step 5. Enqueue the steps to snapshot the lock state for manager with promise to the lock task queue.
        let callback =
            callback_promise(&promise, self, global.task_manager().weblocks_task_source());
        if let Err(e) = self.send_storage_msg(WebLocksThreadMsg::Query(
            callback,
            self.get_immutable_origin(),
        )) {
            warn!("Query failed with {e}");
        }

        // Step 6. Return promise.
        promise
    }
}

impl RoutedPromiseListener<LockManagerSnapshotMsg> for LockManager {
    fn handle_response(
        &self,
        cx: &mut JSContext,
        msg: LockManagerSnapshotMsg,
        promise: &RootedPromise,
    ) {
        let snapshot: LockManagerSnapshot = msg.convert();
        promise.resolve_native(cx, &snapshot);
    }
}

impl Convert<LockManagerSnapshot> for LockManagerSnapshotMsg {
    fn convert(self) -> LockManagerSnapshot {
        let mut snapshot = LockManagerSnapshot::empty();
        snapshot.held = Some(self.held.into_iter().map(Convert::convert).collect());
        snapshot.pending = Some(self.pending.into_iter().map(Convert::convert).collect());
        snapshot
    }
}

impl Convert<LockInfo> for LockInfoMsg {
    fn convert(self) -> LockInfo {
        let mut lock_info = LockInfo::empty();
        lock_info.name = Some(self.name.into());
        lock_info.mode = Some(self.mode.convert());
        lock_info.clientId = Some(self.client_id.into());
        lock_info
    }
}

impl Convert<LockMode> for LockModeMsg {
    fn convert(self) -> LockMode {
        match self {
            LockModeMsg::Shared => LockMode::Shared,
            LockModeMsg::Exclusive => LockMode::Exclusive,
        }
    }
}

impl Convert<LockModeMsg> for LockMode {
    fn convert(self) -> LockModeMsg {
        match self {
            LockMode::Shared => LockModeMsg::Shared,
            LockMode::Exclusive => LockModeMsg::Exclusive,
        }
    }
}
