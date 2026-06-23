use js::context::JSContext;
use js::realm::AutoRealm;
use script_bindings::realms::enter_auto_realm;
use script_bindings::root::DomRoot;
use script_promise::{
    EnqueueWaitForallMicrotask, PromiseGlobalScopeTrait, WaitForAllSuccessStepsMicrotask,
};

use crate::dom::GlobalScope;
use crate::microtask::{Microtask, MicrotaskRunnable};

impl MicrotaskRunnable for WaitForAllSuccessStepsMicrotask<GlobalScope> {
    fn handler(&self, cx: &mut JSContext) {
        (self.success_steps)(cx, vec![]);
    }

    fn enter_realm<'cx>(&self, cx: &'cx mut JSContext) -> AutoRealm<'cx> {
        enter_auto_realm(cx, &*self.global)
    }
}

impl EnqueueWaitForallMicrotask<GlobalScope> for GlobalScope {
    fn enqueue(global: &GlobalScope, task: WaitForAllSuccessStepsMicrotask<GlobalScope>) {
        global.enqueue_microtask(Microtask::WaitForAllSuccessSteps(task));
    }
}

impl PromiseGlobalScopeTrait for GlobalScope {
    fn get_cx() -> script_bindings::script_runtime::JSContext {
        GlobalScope::get_cx()
    }
}
