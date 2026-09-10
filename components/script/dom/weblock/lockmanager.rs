use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::{
    callback::RootedCallback,
    codegen::GenericBindings::WebLockBinding::{
        LockGrantedCallback, LockManagerMethods, LockOptions,
    },
    interfaces::PromiseHelpers,
    reflector::{Reflector, reflect_dom_object_with_cx},
    root::DomRoot,
    str::DOMString,
};
use servo_base::generic_channel::{GenericSend, SendResult};
use storage_traits::weblocks::{LockInfoMsg, LockManagerSnapshotMsg, WebLocksThreadMsg};

use crate::{
    dom::{GlobalScope, Promise, RootedPromise, bindings::reflector::DomGlobal},
    routed_promise::{RoutedPromiseListener, callback_promise},
};

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

    fn request(&self) {
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
        todo!()
    }

    fn Request_(
        &self,
        cx: &mut JSContext,
        name: DOMString,
        options: &LockOptions<crate::DomTypeHolder>,
        callback: RootedCallback<LockGrantedCallback<crate::DomTypeHolder>>,
    ) -> RootedPromise {
        todo!()
    }

    fn Query(&self, cx: &mut JSContext) -> RootedPromise {
        let global = self.global();
        let promise = Promise::new_rooted(cx, &global);
        let task_manager = global.task_manager();
        // TODO: custom task source
        let task_source = task_manager.dom_manipulation_task_source();
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
        response: LockManagerSnapshotMsg,
        promise: &RootedPromise,
    ) {
        let global = self.global();
        todo!()
    }
}
