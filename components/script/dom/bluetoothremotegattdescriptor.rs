/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bluetooth_traits::{BluetoothRequest, BluetoothResponse};
use bluetooth_traits::blacklist::{Blacklist, uuid_is_blacklisted};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BluetoothDeviceBinding::BluetoothDeviceMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTCharacteristicBinding::
    BluetoothRemoteGATTCharacteristicMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTDescriptorBinding;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTDescriptorBinding::BluetoothRemoteGATTDescriptorMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServerBinding::BluetoothRemoteGATTServerMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServiceBinding::BluetoothRemoteGATTServiceMethods;
use dom::bindings::error::Error::{self, InvalidModification, Network, Security};
use dom::bindings::js::{JS, MutHeap, Root};
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
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
    characteristic: MutHeap<JS<BluetoothRemoteGATTCharacteristic>>,
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
            characteristic: MutHeap::new(characteristic),
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
       self.characteristic.get()
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
        if uuid_is_blacklisted(self.uuid.as_ref(), Blacklist::Reads) {
            p.reject_error(p_cx, Security);
            return p;
        }
        if !self.Characteristic().Service().Device().Gatt().Connected() {
            p.reject_error(p_cx, Network);
            return p;
        }
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
        if uuid_is_blacklisted(self.uuid.as_ref(), Blacklist::Writes) {
            p.reject_error(p_cx, Security);
            return p;
        }
        if value.len() > MAXIMUM_ATTRIBUTE_LENGTH {
            p.reject_error(p_cx, InvalidModification);
            return p;
        }
        if !self.Characteristic().Service().Device().Gatt().Connected() {
            p.reject_error(p_cx, Network);
            return p;
        }
        let sender = response_async(&p, self);
        self.get_bluetooth_thread().send(
            BluetoothRequest::WriteValue(self.get_instance_id(), value, sender)).unwrap();
        return p;
    }
}

impl AsyncBluetoothListener for BluetoothRemoteGATTDescriptor {
    fn handle_response(&self, response: BluetoothResponse, promise_cx: *mut JSContext, promise: &Rc<Promise>) {
        match response {
            BluetoothResponse::ReadValue(result) => {
                let value = ByteString::new(result);
                *self.value.borrow_mut() = Some(value.clone());
                promise.resolve_native(promise_cx, &value);
            },
            BluetoothResponse::WriteValue(result) => {
                let value = ByteString::new(result);
                *self.value.borrow_mut() = Some(value.clone());
                promise.resolve_native(promise_cx, &value);
            },
            _ => promise.reject_error(promise_cx, Error::Type("Something went wrong...".to_owned())),
        }
    }
}
