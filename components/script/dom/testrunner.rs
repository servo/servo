/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bluetooth_traits::BluetoothRequest;
use dom::bindings::codegen::Bindings::TestRunnerBinding;
use dom::bindings::codegen::Bindings::TestRunnerBinding::TestRunnerMethods;
use dom::bindings::error::{Error, ErrorResult};
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use ipc_channel::ipc::{self, IpcSender};

// https://webbluetoothcg.github.io/web-bluetooth/tests#test-runner
 #[dom_struct]
pub struct TestRunner {
    reflector_: Reflector,
}

impl TestRunner {
    pub fn new_inherited() -> TestRunner {
        TestRunner {
            reflector_: Reflector::new(),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<TestRunner> {
        reflect_dom_object(Box::new(TestRunner::new_inherited()),
                           global,
                           TestRunnerBinding::Wrap)
    }

    fn get_bluetooth_thread(&self) -> IpcSender<BluetoothRequest> {
        self.global().as_window().bluetooth_thread()
    }
}

impl TestRunnerMethods for TestRunner {
    // https://webbluetoothcg.github.io/web-bluetooth/tests#setBluetoothMockDataSet
    fn SetBluetoothMockDataSet(&self, dataSetName: DOMString) -> ErrorResult {
        let (sender, receiver) = ipc::channel().unwrap();
        self.get_bluetooth_thread().send(BluetoothRequest::Test(String::from(dataSetName), sender)).unwrap();
        match receiver.recv().unwrap().into() {
            Ok(()) => {
                Ok(())
            },
            Err(error) => {
                Err(Error::from(error))
            },
        }
    }
}
