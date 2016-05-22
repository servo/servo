/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServiceBinding;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServiceBinding::BluetoothRemoteGATTServiceMethods;
use dom::bindings::error::Error::Type;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, MutHeap, Root};
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::bluetoothcharacteristicproperties::BluetoothCharacteristicProperties;
use dom::bluetoothdevice::BluetoothDevice;
use dom::bluetoothremotegattcharacteristic::BluetoothRemoteGATTCharacteristic;
use dom::bluetoothuuid::{BluetoothCharacteristicUUID, BluetoothUUID};
use ipc_channel::ipc::{self, IpcSender};
use net_traits::bluetooth_thread::BluetoothMethodMsg;

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattservice
#[dom_struct]
pub struct BluetoothRemoteGATTService {
    reflector_: Reflector,
    device: MutHeap<JS<BluetoothDevice>>,
    uuid: DOMString,
    isPrimary: bool,
    instanceID: String,
}

impl BluetoothRemoteGATTService {
    pub fn new_inherited(device: &BluetoothDevice,
                         uuid: DOMString,
                         isPrimary: bool,
                         instanceID: String)
                         -> BluetoothRemoteGATTService {
        BluetoothRemoteGATTService {
            reflector_: Reflector::new(),
            device: MutHeap::new(device),
            uuid: uuid,
            isPrimary: isPrimary,
            instanceID: instanceID,
        }
    }

    pub fn new(global: GlobalRef,
               device: &BluetoothDevice,
               uuid: DOMString,
               isPrimary: bool,
               instanceID: String)
               -> Root<BluetoothRemoteGATTService> {
        reflect_dom_object(box BluetoothRemoteGATTService::new_inherited(device,
                                                                         uuid,
                                                                         isPrimary,
                                                                         instanceID),
                           global,
                           BluetoothRemoteGATTServiceBinding::Wrap)
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

impl BluetoothRemoteGATTServiceMethods for BluetoothRemoteGATTService {
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-device
    fn Device(&self) -> Root<BluetoothDevice> {
        self.device.get()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-isprimary
    fn IsPrimary(&self) -> bool {
        self.isPrimary
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-uuid
    fn Uuid(&self) -> DOMString {
        self.uuid.clone()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-getcharacteristic
    fn GetCharacteristic(&self,
                         characteristic: BluetoothCharacteristicUUID)
                         -> Fallible<Root<BluetoothRemoteGATTCharacteristic>> {
        let uuid = try!(BluetoothUUID::GetCharacteristic(self.global().r(), characteristic)).to_string();
        let (sender, receiver) = ipc::channel().unwrap();
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::GetCharacteristic(self.get_instance_id(), uuid, sender)).unwrap();
        let characteristic = receiver.recv().unwrap();
        match characteristic {
            Ok(characteristic) => {
                let properties = BluetoothCharacteristicProperties::new(self.global().r(),
                                                                        characteristic.broadcast,
                                                                        characteristic.read,
                                                                        characteristic.write_without_response,
                                                                        characteristic.write,
                                                                        characteristic.notify,
                                                                        characteristic.indicate,
                                                                        characteristic.authenticated_signed_writes,
                                                                        characteristic.reliable_write,
                                                                        characteristic.writable_auxiliaries);
                Ok(BluetoothRemoteGATTCharacteristic::new(self.global().r(),
                                                          self,
                                                          DOMString::from(characteristic.uuid),
                                                          &properties,
                                                          characteristic.instance_id))
            },
            Err(error) => {
                Err(Type(error))
            },
        }
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-getcharacteristics
    fn GetCharacteristics(&self,
                          characteristic: Option<BluetoothCharacteristicUUID>)
                          -> Fallible<Vec<Root<BluetoothRemoteGATTCharacteristic>>> {
        let mut uuid: Option<String> = None;
        if let Some(c) = characteristic {
            uuid = Some(try!(BluetoothUUID::GetCharacteristic(self.global().r(), c)).to_string())
        };
        let mut characteristics = vec!();
        let (sender, receiver) = ipc::channel().unwrap();
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::GetCharacteristics(self.get_instance_id(), uuid, sender)).unwrap();
        let characteristics_vec = receiver.recv().unwrap();
        match characteristics_vec {
            Ok(characteristic_vec) => {
                for characteristic in characteristic_vec {
                    let properties = BluetoothCharacteristicProperties::new(self.global().r(),
                                                                            characteristic.broadcast,
                                                                            characteristic.read,
                                                                            characteristic.write_without_response,
                                                                            characteristic.write,
                                                                            characteristic.notify,
                                                                            characteristic.indicate,
                                                                            characteristic.authenticated_signed_writes,
                                                                            characteristic.reliable_write,
                                                                            characteristic.writable_auxiliaries);
                    characteristics.push(BluetoothRemoteGATTCharacteristic::new(self.global().r(),
                                                                                self,
                                                                                DOMString::from(characteristic.uuid),
                                                                                &properties,
                                                                                characteristic.instance_id));
                }
                Ok(characteristics)
            },
            Err(error) => {
                Err(Type(error))
            },
        }
    }
}
