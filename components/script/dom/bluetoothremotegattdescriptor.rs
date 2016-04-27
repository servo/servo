/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BluetoothDeviceBinding::BluetoothDeviceMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTCharacteristicBinding::
    BluetoothRemoteGATTCharacteristicMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTDescriptorBinding;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTDescriptorBinding::BluetoothRemoteGATTDescriptorMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServerBinding::BluetoothRemoteGATTServerMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServiceBinding::BluetoothRemoteGATTServiceMethods;
use dom::bindings::error::Error::{Type, Network};
use dom::bindings::error::{Fallible, ErrorResult};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, MutHeap, Root};
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::str::ByteString;
use dom::bluetoothremotegattcharacteristic::BluetoothRemoteGATTCharacteristic;
use ipc_channel::ipc::{self, IpcSender};
use net_traits::bluetooth_thread::{BluetoothMethodMsg, BluetoothObjectMsg};
use util::str::DOMString;

// http://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattdescriptor
#[dom_struct]
pub struct BluetoothRemoteGATTDescriptor {
    reflector_: Reflector,
    characteristic: MutHeap<JS<BluetoothRemoteGATTCharacteristic>>,
    uuid: DOMString,
    value: DOMRefCell<Option<ByteString>>,
    instanceID: String,
}

impl BluetoothRemoteGATTDescriptor {
    pub fn new_inherited(characteristic: &BluetoothRemoteGATTCharacteristic,
                         uuid: DOMString,
                         instanceID: String)
                         -> BluetoothRemoteGATTDescriptor {
        BluetoothRemoteGATTDescriptor {
            reflector_: Reflector::new(),
            characteristic: MutHeap::new(characteristic),
            uuid: uuid,
            value: DOMRefCell::new(None),
            instanceID: instanceID,
        }
    }

    pub fn new(global: GlobalRef,
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
        let global_root = self.global();
        let global_ref = global_root.r();
        global_ref.as_window().bluetooth_thread()
    }

    pub fn get_instance_id(&self) -> String {
        self.instanceID.clone()
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

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattdescriptor-readvalue
    fn ReadValue(&self) -> Fallible<ByteString> {
        let (sender, receiver) = ipc::channel().unwrap();
        if !self.Characteristic().Service().Device().Gatt().Connected() {
            return Err(Network)
        }
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::ReadValue(self.get_instance_id(), sender)).unwrap();
        let result = receiver.recv().unwrap();
        let value = match result {
            BluetoothObjectMsg::BluetoothReadValue {
                value
            } => {
                ByteString::new(value)
            },
            BluetoothObjectMsg::Error {
                error
            } => {
                return Err(Type(error))
            },
            _ => unreachable!()
        };
        *self.value.borrow_mut() = Some(value.clone());
        Ok(value)
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattdescriptor-writevalue
    fn WriteValue(&self, value: Vec<u8>) -> ErrorResult {
        let (sender, receiver) = ipc::channel().unwrap();
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::WriteValue(self.get_instance_id(), value, sender)).unwrap();
        let result = receiver.recv().unwrap();
        match result {
            BluetoothObjectMsg::BluetoothWriteValue => Ok(()),
            BluetoothObjectMsg::Error {
                error
            } => {
                Err(Type(error))
            },
            _ => unreachable!()
        }
    }
}
