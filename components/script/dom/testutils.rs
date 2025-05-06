/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::jsapi::{GCReason, JS_GC};
use script_bindings::reflector::Reflector;
use script_bindings::script_runtime::CanGc;

use super::globalscope::GlobalScope;
use crate::dom::bindings::codegen::Bindings::TestUtilsBinding::TestUtilsMethods;
use crate::dom::promise::Promise;
use crate::test::TrustedPromise;

#[dom_struct]
pub(crate) struct TestUtils {
    reflector_: Reflector,
}

impl TestUtilsMethods<crate::DomTypeHolder> for TestUtils {
    /// <https://testutils.spec.whatwg.org/#dom-testutils-gc>
    #[allow(unsafe_code)]
    fn Gc(global: &GlobalScope) -> Rc<Promise> {
        // 1. Let p be a new promise.
        let promise = Promise::new(global, CanGc::note());
        let trusted = TrustedPromise::new(promise.clone());
        // 2. Run the following in parallel:
        // 2.1 Run implementation-defined steps to perform a garbage collection covering at least the entry Realm.
        // 2.2 Resolve p.
        // We need to spin the event-loop in order get the GC to actually run.
        // We do this by queuing a task that calls the GC and then resolves the promise.
        let task = task!(testutils_gc: move || {
            unsafe {
                JS_GC(*GlobalScope::get_cx(), GCReason::DOM_TESTUTILS);
            }
            let promise = trusted.root();
            promise.resolve_native(&(), CanGc::note());
        });

        global
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task);

        promise
    }
}
