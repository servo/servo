/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BluetoothDeviceBinding::BluetoothDeviceMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTCharacteristicBinding;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTCharacteristicBinding::
    BluetoothRemoteGATTCharacteristicMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServerBinding::BluetoothRemoteGATTServerMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServiceBinding::BluetoothRemoteGATTServiceMethods;
use dom::bindings::error::Error::{Network, Type};
use dom::bindings::error::{Fallible, ErrorResult};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, MutHeap, Root};
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::str::ByteString;
use dom::bluetoothcharacteristicproperties::BluetoothCharacteristicProperties;
use dom::bluetoothremotegattdescriptor::BluetoothRemoteGATTDescriptor;
use dom::bluetoothremotegattservice::BluetoothRemoteGATTService;
use dom::bluetoothuuid::{BluetoothDescriptorUUID, BluetoothUUID};
use ipc_channel::ipc::{self, IpcSender};
use net_traits::bluetooth_thread::{BluetoothMethodMsg, BluetoothObjectMsg};
use util::str::DOMString;

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattcharacteristic
#[dom_struct]
pub struct BluetoothRemoteGATTCharacteristic {
    reflector_: Reflector,
    service: MutHeap<JS<BluetoothRemoteGATTService>>,
    uuid: DOMString,
    properties: MutHeap<JS<BluetoothCharacteristicProperties>>,
    value: DOMRefCell<Option<ByteString>>,
    instanceID: String,
}

impl BluetoothRemoteGATTCharacteristic {
    pub fn new_inherited(service: &BluetoothRemoteGATTService,
                         uuid: DOMString,
                         properties: &BluetoothCharacteristicProperties,
                         instanceID: String)
                         -> BluetoothRemoteGATTCharacteristic {
        BluetoothRemoteGATTCharacteristic {
            reflector_: Reflector::new(),
            service: MutHeap::new(service),
            uuid: uuid,
            properties: MutHeap::new(properties),
            value: DOMRefCell::new(None),
            instanceID: instanceID,
        }
    }

    pub fn new(global: GlobalRef,
               service: &BluetoothRemoteGATTService,
               uuid: DOMString,
               properties: &BluetoothCharacteristicProperties,
               instanceID: String)
               -> Root<BluetoothRemoteGATTCharacteristic> {
        reflect_dom_object(box BluetoothRemoteGATTCharacteristic::new_inherited(service,
                                                                                uuid,
                                                                                properties,
                                                                                instanceID),
                           global,
                           BluetoothRemoteGATTCharacteristicBinding::Wrap)
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

impl BluetoothRemoteGATTCharacteristicMethods for BluetoothRemoteGATTCharacteristic {

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-properties
    fn Properties(&self) -> Root<BluetoothCharacteristicProperties> {
        self.properties.get()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-service
    fn Service(&self) -> Root<BluetoothRemoteGATTService> {
        self.service.get()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-uuid
    fn Uuid(&self) -> DOMString {
        self.uuid.clone()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-getdescriptor
    fn GetDescriptor(&self, descriptor: BluetoothDescriptorUUID) -> Fallible<Root<BluetoothRemoteGATTDescriptor>> {
        let uuid: String = match BluetoothUUID::GetDescriptor(self.global().r(), descriptor.clone()) {
            Ok(domstring) => domstring.to_string(),
            Err(error) => return Err(error),
        };
        let (sender, receiver) = ipc::channel().unwrap();
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::GetDescriptor(self.get_instance_id(), uuid, sender)).unwrap();
        let descriptor = receiver.recv().unwrap();
        match descriptor {
            BluetoothObjectMsg::BluetoothDescriptor {
                uuid,
                instance_id
            } => {
                Ok(BluetoothRemoteGATTDescriptor::new(self.global().r(),
                                                      &self,
                                                      DOMString::from(uuid),
                                                      instance_id))
            },
            BluetoothObjectMsg::Error {
                error
            } => {
                Err(Type(error))
            },
            _ => unreachable!()
        }
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-getdescriptors
    fn GetDescriptors(&self,
                      descriptor: Option<BluetoothDescriptorUUID>)
                      -> Fallible<Vec<Root<BluetoothRemoteGATTDescriptor>>> {
        let mut uuid: Option<String> = None;
        if let Some(d)= descriptor {
            match BluetoothUUID::GetCharacteristic(self.global().r(), d.clone()) {
                Ok(domstring) => uuid = Some(domstring.to_string()),
                Err(error) => return Err(error),
            }
        };
        let (sender, receiver) = ipc::channel().unwrap();
        let mut descriptors: Vec<Root<BluetoothRemoteGATTDescriptor>> = vec!();
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::GetDescriptors(self.get_instance_id(), uuid, sender)).unwrap();
        let descriptors_vec = receiver.recv().unwrap();
        match descriptors_vec {
            BluetoothObjectMsg::BluetoothDescriptors {
                descriptors_vec
            } => {
                for d in descriptors_vec {
                    match d {
                        BluetoothObjectMsg::BluetoothDescriptor {
                            uuid,
                            instance_id,
                        } => {
                            descriptors.push(BluetoothRemoteGATTDescriptor::new(self.global().r(),
                                                                                &self,
                                                                                DOMString::from(uuid),
                                                                                instance_id));
                        },
                        _ => unreachable!(),
                    }
                }
                Ok(descriptors)
            },
            BluetoothObjectMsg::Error {
                error
            } => {
                Err(Type(error))
            },
            _ => unreachable!(),
        }
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-value
    fn GetValue(&self) -> Option<ByteString> {
        self.value.borrow().clone()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-readvalue
    fn ReadValue(&self) -> Fallible<ByteString> {
        let (sender, receiver) = ipc::channel().unwrap();
        if !self.Service().Device().Gatt().Connected() {
            Err(Network)
        } else {
            self.get_bluetooth_thread().send(
                BluetoothMethodMsg::ReadValue(self.get_instance_id(), sender)).unwrap();
            let result = receiver.recv().unwrap();
            let value = match result {
                BluetoothObjectMsg::BluetoothReadValue {
                    value
                } => {
                    Some(ByteString::new(value))
                },
                BluetoothObjectMsg::Error {
                    error
                } => {
                    return Err(Type(error))
                },
                _ => unreachable!()
            };
            *self.value.borrow_mut() = value;
            Ok(self.GetValue().unwrap())
        }
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-writevalue
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
