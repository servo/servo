/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line
use std::rc::Rc;

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::TestWorkletBinding::TestWorkletMethods;
use crate::dom::bindings::codegen::Bindings::WorkletBinding::Worklet_Binding::WorkletMethods;
use crate::dom::bindings::codegen::Bindings::WorkletBinding::WorkletOptions;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::promise::Promise;
use crate::dom::window::Window;
use crate::dom::worklet::Worklet;
use crate::dom::workletglobalscope::WorkletGlobalScopeType;
use crate::realms::InRealm;
use crate::script_runtime::CanGc;
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

    fn new(window: &Window, proto: Option<HandleObject>, can_gc: CanGc) -> DomRoot<TestWorklet> {
        let worklet = Worklet::new(window, WorkletGlobalScopeType::Test, can_gc);
        reflect_dom_object_with_proto(
            Box::new(TestWorklet::new_inherited(&worklet)),
            window,
            proto,
            can_gc,
        )
    }
}

impl TestWorkletMethods<crate::DomTypeHolder> for TestWorklet {
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<TestWorklet>> {
        Ok(TestWorklet::new(window, proto, can_gc))
    }

    #[allow(non_snake_case)]
    fn AddModule(
        &self,
        moduleURL: USVString,
        options: &WorkletOptions,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        self.worklet.AddModule(moduleURL, options, comp, can_gc)
    }

    fn Lookup(&self, key: DOMString) -> Option<DOMString> {
        let id = self.worklet.worklet_id();
        let pool = ScriptThread::worklet_thread_pool();
        pool.test_worklet_lookup(id, String::from(key))
            .map(DOMString::from)
    }
}
