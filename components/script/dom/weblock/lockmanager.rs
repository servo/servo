use dom_struct::dom_struct;
use js::context::JSContext;
use js::jsval::UndefinedValue;
use script_bindings::callback::RootedCallback;
use script_bindings::codegen::GenericBindings::AbortSignalBinding::AbortSignalMethods;
use script_bindings::codegen::GenericBindings::WebLockBinding::{
    LockGrantedCallback, LockManagerMethods, LockMode, LockOptions,
};
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use script_bindings::root::{Dom, DomRoot, Root};
use script_bindings::str::DOMString;
use servo_base::generic_channel::{GenericCallback, GenericSend, SendResult};
use servo_url::ImmutableOrigin;
use storage_traits::weblocks::{
    LockInfoMsg, LockManagerSnapshotMsg, LockModeMsg, LockMsg, LockRequest, WebLocksThreadMsg,
};

use crate::conversions::Convert;
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
        // TODO: request actually involves two promises, fix this later:
        // 1. the query respond, and run callback
        // 2. when callback finish, release the lock
        // 3. when release done, then the final promise, not the "Request" one

        let global = self.global();
        let task_manager = global.task_manager();
        let task_source = task_manager.weblocks_task_source();

        let callback = GenericCallback::new(|_| {
            // TODO
        })
        .unwrap();

        let mut reject_not_supported = |message: &str| {
            Promise::new_rejected_rooted(
                cx,
                &global,
                DOMException::new_inherited(message.into(), "NotSupportedError".into()),
            )
        };

        // Step 1.
        let default_options = LockOptions::default();
        let options = options.unwrap_or(&default_options);

        // Step 3. TODO: check fully active

        // Step 4. skip, manager in storage threads

        // Step 5.
        if name.starts_with('-') {
            return reject_not_supported("name starts with U+002D HYPHEN-MINUS(-)");
        }
        // Step 6.
        if options.steal && options.ifAvailable {
            return reject_not_supported("both steal and ifAvailable are true");
        }
        // Step 7.
        if options.steal && options.mode != LockMode::Exclusive {
            return reject_not_supported("steal is true but mode is not exclusive");
        }
        // Step 8.
        if options.signal.is_some() && (options.steal || options.ifAvailable) {
            return reject_not_supported("signal exists but either steal or ifAvailable is true");
        }
        // Step 9.
        if let Some(signal) = &options.signal
            && signal.aborted()
        {
            rooted!(&in(cx) let mut reason = UndefinedValue());
            signal.Reason(reason.handle_mut());
            return Promise::new_rejected_rooted(cx, &global, *reason);
        }

        // Step 10.
        let promise = Promise::new_rooted(cx, &global);
        // Step 11.
        self.request_lock(
            &promise,
            // TODO: environment id is not implemented in servo, except for service worker
            String::new(),
            callback,
            name,
            options.mode,
            options.ifAvailable,
            options.steal,
            &options.signal,
        );
        // Step 12.
        promise
    }

    /// <https://www.w3.org/TR/web-locks/#request-a-lock>
    #[expect(clippy::too_many_arguments)]
    fn request_lock(
        &self,
        promise: &RootedPromise,
        client_id: String,
        callback: GenericCallback<LockMsg>,
        name: DOMString,
        mode: LockMode,
        if_available: bool,
        steal: bool,
        signal: &Option<Root<Dom<AbortSignal>>>,
    ) {
        // Step 1.
        let request = LockRequest {
            client_id,
            name: name.into(),
            mode: mode.convert(),
            callback,
            if_available,
            steal,
        };

        // Step 2-4 happen on storage_threads
        if let Err(e) = self.send_storage_msg(WebLocksThreadMsg::Request(
            request,
            self.get_immutable_origin(),
        )) {
            warn!("Request failed with {e}");
        }

        // TODO: enqueue abort
        // TODO: resolve promise when release/abort
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
        let global = self.global();
        let promise = Promise::new_rooted(cx, &global);
        let task_manager = global.task_manager();
        let task_source = task_manager.weblocks_task_source();
        let callback = callback_promise(&promise, self, task_source);

        if let Err(e) = self.send_storage_msg(WebLocksThreadMsg::Query(
            callback,
            self.get_immutable_origin(),
        )) {
            warn!("Query failed with {e}");
        }

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
