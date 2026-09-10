use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::callback::RootedCallback;
use script_bindings::codegen::GenericBindings::WebLockBinding::{
    LockGrantedCallback, LockManagerMethods, LockMode, LockOptions,
};
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use script_bindings::root::DomRoot;
use script_bindings::str::DOMString;
use servo_base::generic_channel::{GenericSend, SendResult};
use storage_traits::weblocks::{
    LockInfoMsg, LockManagerSnapshotMsg, LockModeMsg, WebLocksThreadMsg,
};

use crate::conversions::Convert;
use crate::dom::bindings::codegen::Bindings::WebLockBinding::{LockInfo, LockManagerSnapshot};
use crate::dom::bindings::reflector::DomGlobal;
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

    fn request(
        &self,
        cx: &mut JSContext,
        name: DOMString,
        options: Option<&LockOptions<crate::DomTypeHolder>>,
        callback: RootedCallback<LockGrantedCallback<crate::DomTypeHolder>>,
    ) -> RootedPromise {
        todo!()
        // self.
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

        if let Err(e) = self.send_storage_msg(WebLocksThreadMsg::Query(callback)) {
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
        let global = self.global();

        let snapshot: LockManagerSnapshot = msg.convert();

        todo!()
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
            LockModeMsg::Exclusive => LockMode::Exclusive,
            LockModeMsg::Shared => LockMode::Shared,
        }
    }
}
