/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// check-tidy: no specs after this line

use dom::bindings::codegen::Bindings::TestWorkletBinding::TestWorkletMethods;
use dom::bindings::codegen::Bindings::TestWorkletBinding::Wrap;
use dom::bindings::codegen::Bindings::WorkletBinding::WorkletBinding::WorkletMethods;
use dom::bindings::codegen::Bindings::WorkletBinding::WorkletOptions;
use dom::bindings::error::Fallible;
use dom::bindings::reflector::Reflector;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::{Dom, DomRoot};
use dom::bindings::str::DOMString;
use dom::bindings::str::USVString;
use dom::promise::Promise;
use dom::window::Window;
use dom::worklet::Worklet;
use dom::worklet::WorkletThreadPool;
use dom::workletglobalscope::WorkletGlobalScopeType;
use dom_struct::dom_struct;
use script_thread::ScriptThread;
use std::rc::Rc;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct TestWorklet<TH: TypeHolderTrait> {
    reflector: Reflector<TH>,
    worklet: Dom<Worklet<TH>>,
}

impl<TH: TypeHolderTrait> TestWorklet<TH> {
    fn new_inherited(worklet: &Worklet<TH>) -> TestWorklet<TH> {
        TestWorklet {
            reflector: Reflector::new(),
            worklet: Dom::from_ref(worklet),
        }
    }

    fn new(window: &Window<TH>) -> DomRoot<TestWorklet<TH>> {
        let worklet = Worklet::new(window, WorkletGlobalScopeType::Test);
        reflect_dom_object(Box::new(TestWorklet::new_inherited(&*worklet)), window, Wrap)
    }

    pub fn Constructor(window: &Window<TH>) -> Fallible<DomRoot<TestWorklet<TH>>> {
        Ok(TestWorklet::new(window))
    }
}

impl<TH: TypeHolderTrait> TestWorkletMethods<TH> for TestWorklet<TH> {
    #[allow(unrooted_must_root)]
    fn AddModule(&self, moduleURL: USVString, options: &WorkletOptions) -> Rc<Promise<TH>> {
        self.worklet.AddModule(moduleURL, options)
    }

    fn Lookup(&self, key: DOMString) -> Option<DOMString> {
        let id = self.worklet.worklet_id();
        let pool: Rc<WorkletThreadPool<TH>> = ScriptThread::worklet_thread_pool();
        pool.test_worklet_lookup(id, String::from(key)).map(DOMString::from)
    }
}
