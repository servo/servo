use js::context::JSContext;
use js::realm::AutoRealm;
use script_bindings::realms::enter_auto_realm;
use script_promise::{EnqueueWaitForallMicrotask, WaitForAllSuccessStepsMicrotask};

use crate::dom::GlobalScope;
use crate::microtask::{Microtask, MicrotaskRunnable};

impl MicrotaskRunnable for WaitForAllSuccessStepsMicrotask<crate::DomTypeHolder> {
    fn handler(&self, cx: &mut JSContext) {
        (self.success_steps)(cx, vec![]);
    }

    fn enter_realm<'cx>(&self, cx: &'cx mut JSContext) -> AutoRealm<'cx> {
        enter_auto_realm::<crate::DomTypeHolder>(cx, &*self.global)
    }
}

impl EnqueueWaitForallMicrotask<crate::DomTypeHolder> for GlobalScope {
    fn enqueue(global: &GlobalScope, task: WaitForAllSuccessStepsMicrotask<crate::DomTypeHolder>) {
        global.enqueue_microtask(Microtask::WaitForAllSuccessSteps(task));
    }
}
