/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line

use crate::dom::bindings::codegen::Bindings::TestWorkletBinding::TestWorkletMethods;
use crate::dom::bindings::codegen::Bindings::TestWorkletBinding::Wrap;
use crate::dom::bindings::codegen::Bindings::WorkletBinding::WorkletBinding::WorkletMethods;
use crate::dom::bindings::codegen::Bindings::WorkletBinding::WorkletOptions;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::reflector::Reflector;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::str::USVString;
use crate::dom::promise::Promise;
use crate::dom::window::Window;
use crate::dom::worklet::Worklet;
use crate::dom::workletglobalscope::WorkletGlobalScopeType;
use crate::script_thread::ScriptThread;
use dom_struct::dom_struct;
use std::rc::Rc;

#[dom_struct]
pub struct TestWorklet {
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

    fn new(window: &Window) -> DomRoot<TestWorklet> {
        let worklet = Worklet::new(window, WorkletGlobalScopeType::Test);
        reflect_dom_object(
            Box::new(TestWorklet::new_inherited(&*worklet)),
            window,
            Wrap,
        )
    }

    pub fn Constructor(window: &Window) -> Fallible<DomRoot<TestWorklet>> {
        Ok(TestWorklet::new(window))
    }
}

impl TestWorkletMethods for TestWorklet {
    fn AddModule(&self, moduleURL: USVString, options: &WorkletOptions) -> Rc<Promise> {
        self.worklet.AddModule(moduleURL, options)
    }

    fn Lookup(&self, key: DOMString) -> Option<DOMString> {
        let id = self.worklet.worklet_id();
        let pool = ScriptThread::worklet_thread_pool();
        pool.test_worklet_lookup(id, String::from(key))
            .map(DOMString::from)
    }
}
