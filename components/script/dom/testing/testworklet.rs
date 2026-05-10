/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line
use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::realm::CurrentRealm;
use js::rust::HandleObject;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_proto_and_cx};

use crate::dom::bindings::codegen::Bindings::TestWorkletBinding::TestWorkletMethods;
use crate::dom::bindings::codegen::Bindings::WorkletBinding::Worklet_Binding::WorkletMethods;
use crate::dom::bindings::codegen::Bindings::WorkletBinding::WorkletOptions;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::promise::Promise;
use crate::dom::window::Window;
use crate::dom::worklet::Worklet;
use crate::dom::workletglobalscope::WorkletGlobalScopeType;
use crate::script_thread::ScriptThread;

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
        let worklet = Worklet::new(cx, window, WorkletGlobalScopeType::Test);
        reflect_dom_object_with_proto_and_cx(
            Box::new(TestWorklet::new_inherited(&worklet)),
            window,
            proto,
            cx,
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
        let pool = ScriptThread::worklet_thread_pool(self.global().image_cache());
        pool.test_worklet_lookup(id, String::from(key))
            .map(DOMString::from)
    }
}
