/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bluetooth_traits::{BluetoothRequest, BluetoothResponse};
use bluetooth_traits::blocklist::{Blocklist, uuid_is_blocklisted};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BluetoothDeviceBinding::BluetoothDeviceMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTCharacteristicBinding::
    BluetoothRemoteGATTCharacteristicMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTDescriptorBinding;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTDescriptorBinding::BluetoothRemoteGATTDescriptorMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServerBinding::BluetoothRemoteGATTServerMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServiceBinding::BluetoothRemoteGATTServiceMethods;
use dom::bindings::error::Error::{self, InvalidModification, Network, Security};
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::str::{ByteString, DOMString};
use dom::bluetooth::{AsyncBluetoothListener, response_async};
use dom::bluetoothremotegattcharacteristic::{BluetoothRemoteGATTCharacteristic, MAXIMUM_ATTRIBUTE_LENGTH};
use dom::globalscope::GlobalScope;
use dom::promise::Promise;
use ipc_channel::ipc::IpcSender;
use js::jsapi::JSContext;
use std::rc::Rc;

// http://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattdescriptor
#[dom_struct]
pub struct BluetoothRemoteGATTDescriptor {
    reflector_: Reflector,
    characteristic: JS<BluetoothRemoteGATTCharacteristic>,
    uuid: DOMString,
    value: DOMRefCell<Option<ByteString>>,
    instance_id: String,
}

impl BluetoothRemoteGATTDescriptor {
    pub fn new_inherited(characteristic: &BluetoothRemoteGATTCharacteristic,
                         uuid: DOMString,
                         instance_id: String)
                         -> BluetoothRemoteGATTDescriptor {
        BluetoothRemoteGATTDescriptor {
            reflector_: Reflector::new(),
            characteristic: JS::from_ref(characteristic),
            uuid: uuid,
            value: DOMRefCell::new(None),
            instance_id: instance_id,
        }
    }

    pub fn new(global: &GlobalScope,
               characteristic: &BluetoothRemoteGATTCharacteristic,
               uuid: DOMString,
               instanceID: String)
               -> Root<BluetoothRemoteGATTDescriptor>{
        reflect_dom_object(box BluetoothRemoteGATTDescriptor::new_inherited(characteristic,
                                                                            uuid,
                                                                            instanceID),
                            global,
                            BluetoothRemoteGATTDescriptorBinding::Wrap)
    }

    fn get_bluetooth_thread(&self) -> IpcSender<BluetoothRequest> {
        self.global().as_window().bluetooth_thread()
    }

    fn get_instance_id(&self) -> String {
        self.instance_id.clone()
    }
}

impl BluetoothRemoteGATTDescriptorMethods for BluetoothRemoteGATTDescriptor {
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattdescriptor-characteristic
    fn Characteristic(&self) -> Root<BluetoothRemoteGATTCharacteristic> {
       Root::from_ref(&self.characteristic)
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattdescriptor-uuid
    fn Uuid(&self) -> DOMString {
        self.uuid.clone()
    }

     // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattdescriptor-value
    fn GetValue(&self) -> Option<ByteString> {
        self.value.borrow().clone()
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattdescriptor-readvalue
    fn ReadValue(&self) -> Rc<Promise> {
        let p = Promise::new(&self.global());
        let p_cx = p.global().get_cx();

        // Step 1.
        if uuid_is_blocklisted(self.uuid.as_ref(), Blocklist::Reads) {
            p.reject_error(p_cx, Security);
            return p;
        }

        // Step 2.
        if !self.Characteristic().Service().Device().Gatt().Connected() {
            p.reject_error(p_cx, Network);
            return p;
        }

        // TODO: Step 5: Implement the `connection-checking-wrapper` algorithm for BluetoothRemoteGATTServer.
        // Note: Steps 3 - 4 and substeps of Step 5 are implemented in components/bluetooth/lib.rs
        // in readValue function and in handle_response function.
        let sender = response_async(&p, self);
        self.get_bluetooth_thread().send(
            BluetoothRequest::ReadValue(self.get_instance_id(), sender)).unwrap();
        return p;
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattdescriptor-writevalue
    fn WriteValue(&self, value: Vec<u8>) -> Rc<Promise> {
        let p = Promise::new(&self.global());
        let p_cx = p.global().get_cx();

        // Step 1.
        if uuid_is_blocklisted(self.uuid.as_ref(), Blocklist::Writes) {
            p.reject_error(p_cx, Security);
            return p;
        }

        // Step 2 - 3.
        if value.len() > MAXIMUM_ATTRIBUTE_LENGTH {
            p.reject_error(p_cx, InvalidModification);
            return p;
        }

        // Step 4.
        if !self.Characteristic().Service().Device().Gatt().Connected() {
            p.reject_error(p_cx, Network);
            return p;
        }

        // TODO: Step 7: Implement the `connection-checking-wrapper` algorithm for BluetoothRemoteGATTServer.
        // Note: Steps 5 - 6 and substeps of Step 7 are implemented in components/bluetooth/lib.rs
        // in writeValue function and in handle_response function.
        let sender = response_async(&p, self);
        self.get_bluetooth_thread().send(
            BluetoothRequest::WriteValue(self.get_instance_id(), value, sender)).unwrap();
        return p;
    }
}

impl AsyncBluetoothListener for BluetoothRemoteGATTDescriptor {
    fn handle_response(&self, response: BluetoothResponse, promise_cx: *mut JSContext, promise: &Rc<Promise>) {
        match response {
            // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattdescriptor-readvalue
            BluetoothResponse::ReadValue(result) => {
                // TODO: Step 5.4.1: Implement activeAlgorithms internal slot for BluetoothRemoteGATTServer.

                // Step 5.4.2.
                // TODO(#5014): Replace ByteString with ArrayBuffer when it is implemented.
                let value = ByteString::new(result);
                *self.value.borrow_mut() = Some(value.clone());

                // Step 5.4.3.
                promise.resolve_native(promise_cx, &value);
            },
            // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattdescriptor-writevalue
            BluetoothResponse::WriteValue(result) => {
                // TODO: Step 7.4.1: Implement activeAlgorithms internal slot for BluetoothRemoteGATTServer.

                // Step 7.4.2.
                // TODO(#5014): Replace ByteString with an ArrayBuffer wrapped in a DataView.
                *self.value.borrow_mut() = Some(ByteString::new(result));

                // Step 7.4.3.
                // TODO: Resolve promise with undefined instead of a value.
                promise.resolve_native(promise_cx, &());
            },
            _ => promise.reject_error(promise_cx, Error::Type("Something went wrong...".to_owned())),
        }
    }
}
