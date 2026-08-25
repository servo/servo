/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use profile_traits::generic_channel;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use servo_base::generic_channel::GenericSender;
use servo_bluetooth_traits::BluetoothRequest;

use crate::conversions::Convert;
use crate::dom::bindings::codegen::Bindings::TestRunnerBinding::TestRunnerMethods;
use crate::dom::bindings::error::ErrorResult;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;

// https://webbluetoothcg.github.io/web-bluetooth/tests#test-runner
#[dom_struct]
pub(crate) struct TestRunner {
    reflector_: Reflector,
}

impl TestRunner {
    pub(crate) fn new_inherited() -> TestRunner {
        TestRunner {
            reflector_: Reflector::new(),
        }
    }

    pub(crate) fn new(cx: &mut JSContext, global: &GlobalScope) -> DomRoot<TestRunner> {
        reflect_dom_object_with_cx(Box::new(TestRunner::new_inherited()), global, cx)
    }

    fn get_bluetooth_thread(&self) -> GenericSender<BluetoothRequest> {
        self.global().as_window().bluetooth_thread()
    }
}

impl TestRunnerMethods<crate::DomTypeHolder> for TestRunner {
    // https://webbluetoothcg.github.io/web-bluetooth/tests#setBluetoothMockDataSet
    #[expect(non_snake_case)]
    fn SetBluetoothMockDataSet(&self, dataSetName: DOMString) -> ErrorResult {
        let (sender, receiver) =
            generic_channel::channel(self.global().time_profiler_chan().clone()).unwrap();
        self.get_bluetooth_thread()
            .send(BluetoothRequest::Test(String::from(dataSetName), sender))
            .unwrap();
        match receiver.recv().unwrap() {
            Ok(()) => Ok(()),
            Err(error) => Err(error.convert()),
        }
    }
}
