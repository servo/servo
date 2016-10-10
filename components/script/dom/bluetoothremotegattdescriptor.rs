/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bluetooth_traits::BluetoothMethodMsg;
use bluetooth_traits::blacklist::{Blacklist, uuid_is_blacklisted};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BluetoothDeviceBinding::BluetoothDeviceMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTCharacteristicBinding::
    BluetoothRemoteGATTCharacteristicMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTDescriptorBinding;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTDescriptorBinding::BluetoothRemoteGATTDescriptorMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServerBinding::BluetoothRemoteGATTServerMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServiceBinding::BluetoothRemoteGATTServiceMethods;
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::error::Error::{self, InvalidModification, Network, Security};
use dom::bindings::js::{JS, MutHeap, Root};
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::str::{ByteString, DOMString};
use dom::bluetooth::result_to_promise;
use dom::bluetoothremotegattcharacteristic::{BluetoothRemoteGATTCharacteristic, MAXIMUM_ATTRIBUTE_LENGTH};
use dom::globalscope::GlobalScope;
use dom::promise::Promise;
use ipc_channel::ipc::{self, IpcSender};
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

    fn get_bluetooth_thread(&self) -> IpcSender<BluetoothMethodMsg> {
        self.global().as_window().bluetooth_thread()
    }

    fn get_instance_id(&self) -> String {
        self.instance_id.clone()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattdescriptor-readvalue
    fn read_value(&self) -> Fallible<ByteString> {
        if uuid_is_blacklisted(self.uuid.as_ref(), Blacklist::Reads) {
            return Err(Security)
        }
        let (sender, receiver) = ipc::channel().unwrap();
        if !self.Characteristic().Service().Device().Gatt().Connected() {
            return Err(Network)
        }
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::ReadValue(self.get_instance_id(), sender)).unwrap();
        let result = receiver.recv().unwrap();
        let value = match result {
            Ok(val) => {
                ByteString::new(val)
            },
            Err(error) => {
                return Err(Error::from(error))
            },
        };
        *self.value.borrow_mut() = Some(value.clone());
        Ok(value)
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattdescriptor-writevalue
    fn write_value(&self, value: Vec<u8>) -> ErrorResult {
        if uuid_is_blacklisted(self.uuid.as_ref(), Blacklist::Writes) {
            return Err(Security)
        }
        if value.len() > MAXIMUM_ATTRIBUTE_LENGTH {
            return Err(InvalidModification)
        }
        if !self.Characteristic().Service().Device().Gatt().Connected() {
            return Err(Network)
        }
        let (sender, receiver) = ipc::channel().unwrap();
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::WriteValue(self.get_instance_id(), value.clone(), sender)).unwrap();
        let result = receiver.recv().unwrap();
        match result {
            Ok(_) => Ok(*self.value.borrow_mut() = Some(ByteString::new(value))),
            Err(error) => {
                Err(Error::from(error))
            },
        }
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
        result_to_promise(&self.global(), self.read_value())
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattdescriptor-writevalue
    fn WriteValue(&self, value: Vec<u8>) -> Rc<Promise> {
        result_to_promise(&self.global(), self.write_value(value))
    }
}
