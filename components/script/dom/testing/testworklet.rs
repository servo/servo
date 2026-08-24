/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line
use std::rc::Rc;

use crossbeam_channel::unbounded;
use dom_struct::dom_struct;
use js::context::JSContext;
use js::realm::CurrentRealm;
use js::rust::HandleObject;
use script_bindings::inheritance::Castable;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_proto};

use crate::dom::StatelessWorkletThreadPool;
use crate::dom::bindings::codegen::Bindings::TestWorkletBinding::TestWorkletMethods;
use crate::dom::bindings::codegen::Bindings::WorkletBinding::Worklet_Binding::WorkletMethods;
use crate::dom::bindings::codegen::Bindings::WorkletBinding::WorkletOptions;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::promise::Promise;
use crate::dom::types::{TestWorkletGlobalScope, WorkletGlobalScope};
use crate::dom::window::Window;
use crate::dom::worklet::Worklet;
use crate::dom::workletglobalscope::WorkletGlobalScopeType;

#[dom_struct]
pub(crate) struct TestWorklet {
    reflector: Reflector,
    worklet: Dom<Worklet>,
}

impl TestWorklet {
    fn new_inherited(worklet: &Worklet) -> TestWorklet {
        TestWorklet {
            reflector: Reflector::new(),
            worklet: Dom::from_ref(worklet),
        }
    }

    fn new(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
    ) -> DomRoot<TestWorklet> {
        let worklet_global_scope_init = window.into();
        let worklet = Worklet::new(
            cx,
            window,
            WorkletGlobalScopeType::Test,
            Box::new(|| Rc::new(StatelessWorkletThreadPool::spawn(worklet_global_scope_init))),
        );
        reflect_dom_object_with_proto(
            cx,
            Box::new(TestWorklet::new_inherited(&worklet)),
            window,
            proto,
        )
    }
}

impl TestWorkletMethods<crate::DomTypeHolder> for TestWorklet {
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
    ) -> Fallible<DomRoot<TestWorklet>> {
        Ok(TestWorklet::new(cx, window, proto))
    }

    fn AddModule(
        &self,
        realm: &mut CurrentRealm,
        module_url: USVString,
        options: &WorkletOptions,
    ) -> Rc<Promise> {
        self.worklet.AddModule(realm, module_url, options)
    }

    fn Lookup(&self, key: DOMString) -> Option<DOMString> {
        let id = self.worklet.worklet_id();

        let (sender, receiver) = unbounded();
        let key = String::from(key);

        let lookup_task = move |_cx: &mut JSContext, global_scope: &WorkletGlobalScope| {
            let test_worklet_global_scope = global_scope
                .downcast::<TestWorkletGlobalScope>()
                .expect("TestWorklet's task should be run only on TestWorkletGlobalScope.");
            let value = test_worklet_global_scope.lookup_value(&key);
            let _ = sender.send(value);
        };

        self.worklet
            .worklet_thread_pool()
            .perform_a_worklet_task(id, Box::new(lookup_task));

        match receiver.recv() {
            Ok(value) => value.map(DOMString::from),
            Err(err) => {
                error!("Test Worklet died? {}", err);
                None
            },
        }
    }
}
