/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bluetooth_blacklist::{Blacklist, uuid_is_blacklisted};
use dom::bindings::codegen::Bindings::BluetoothDeviceBinding::BluetoothDeviceMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServerBinding::BluetoothRemoteGATTServerMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServiceBinding;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServiceBinding::BluetoothRemoteGATTServiceMethods;
use dom::bindings::error::Error::{self, Network, Security};
use dom::bindings::error::Fallible;
use dom::bindings::js::{JS, MutHeap, Root};
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::bluetooth::result_to_promise;
use dom::bluetoothcharacteristicproperties::BluetoothCharacteristicProperties;
use dom::bluetoothdevice::BluetoothDevice;
use dom::bluetoothremotegattcharacteristic::BluetoothRemoteGATTCharacteristic;
use dom::bluetoothuuid::{BluetoothCharacteristicUUID, BluetoothServiceUUID, BluetoothUUID};
use dom::globalscope::GlobalScope;
use dom::promise::Promise;
use ipc_channel::ipc::{self, IpcSender};
use net_traits::bluetooth_thread::BluetoothMethodMsg;
use std::rc::Rc;

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattservice
#[dom_struct]
pub struct BluetoothRemoteGATTService {
    reflector_: Reflector,
    device: MutHeap<JS<BluetoothDevice>>,
    uuid: DOMString,
    is_primary: bool,
    instance_id: String,
}

impl BluetoothRemoteGATTService {
    pub fn new_inherited(device: &BluetoothDevice,
                         uuid: DOMString,
                         is_primary: bool,
                         instance_id: String)
                         -> BluetoothRemoteGATTService {
        BluetoothRemoteGATTService {
            reflector_: Reflector::new(),
            device: MutHeap::new(device),
            uuid: uuid,
            is_primary: is_primary,
            instance_id: instance_id,
        }
    }

    pub fn new(global: &GlobalScope,
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
        self.global().as_window().bluetooth_thread()
    }

    fn get_instance_id(&self) -> String {
        self.instance_id.clone()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-getcharacteristic
    fn get_characteristic(&self,
                          characteristic: BluetoothCharacteristicUUID)
                          -> Fallible<Root<BluetoothRemoteGATTCharacteristic>> {
        let uuid = try!(BluetoothUUID::characteristic(characteristic)).to_string();
        if uuid_is_blacklisted(uuid.as_ref(), Blacklist::All) {
            return Err(Security)
        }
        if !self.Device().Gatt().Connected() {
            return Err(Network)
        }
        let (sender, receiver) = ipc::channel().unwrap();
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::GetCharacteristic(self.get_instance_id(), uuid, sender)).unwrap();
        let characteristic = receiver.recv().unwrap();
        match characteristic {
            Ok(characteristic) => {
                let global = self.global();
                let properties = BluetoothCharacteristicProperties::new(&global,
                                                                        characteristic.broadcast,
                                                                        characteristic.read,
                                                                        characteristic.write_without_response,
                                                                        characteristic.write,
                                                                        characteristic.notify,
                                                                        characteristic.indicate,
                                                                        characteristic.authenticated_signed_writes,
                                                                        characteristic.reliable_write,
                                                                        characteristic.writable_auxiliaries);
                Ok(BluetoothRemoteGATTCharacteristic::new(&global,
                                                          self,
                                                          DOMString::from(characteristic.uuid),
                                                          &properties,
                                                          characteristic.instance_id))
            },
            Err(error) => {
                Err(Error::from(error))
            },
        }
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-getcharacteristics
    fn get_characteristics(&self,
                           characteristic: Option<BluetoothCharacteristicUUID>)
                           -> Fallible<Vec<Root<BluetoothRemoteGATTCharacteristic>>> {
        let mut uuid: Option<String> = None;
        if let Some(c) = characteristic {
            uuid = Some(try!(BluetoothUUID::characteristic(c)).to_string());
            if let Some(ref uuid) = uuid {
                if uuid_is_blacklisted(uuid.as_ref(), Blacklist::All) {
                    return Err(Security)
                }
            }
        };
        if !self.Device().Gatt().Connected() {
            return Err(Network)
        }
        let mut characteristics = vec!();
        let (sender, receiver) = ipc::channel().unwrap();
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::GetCharacteristics(self.get_instance_id(), uuid, sender)).unwrap();
        let characteristics_vec = receiver.recv().unwrap();
        match characteristics_vec {
            Ok(characteristic_vec) => {
                for characteristic in characteristic_vec {
                    let global = self.global();
                    let properties = BluetoothCharacteristicProperties::new(&global,
                                                                            characteristic.broadcast,
                                                                            characteristic.read,
                                                                            characteristic.write_without_response,
                                                                            characteristic.write,
                                                                            characteristic.notify,
                                                                            characteristic.indicate,
                                                                            characteristic.authenticated_signed_writes,
                                                                            characteristic.reliable_write,
                                                                            characteristic.writable_auxiliaries);
                    characteristics.push(BluetoothRemoteGATTCharacteristic::new(&global,
                                                                                self,
                                                                                DOMString::from(characteristic.uuid),
                                                                                &properties,
                                                                                characteristic.instance_id));
                }
                Ok(characteristics)
            },
            Err(error) => {
                Err(Error::from(error))
            },
        }
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-getincludedservice
    fn get_included_service(&self,
                           service: BluetoothServiceUUID)
                           -> Fallible<Root<BluetoothRemoteGATTService>> {
        let uuid = try!(BluetoothUUID::service(service)).to_string();
        if uuid_is_blacklisted(uuid.as_ref(), Blacklist::All) {
            return Err(Security)
        }
        if !self.Device().Gatt().Connected() {
            return Err(Network)
        }
        let (sender, receiver) = ipc::channel().unwrap();
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::GetIncludedService(self.get_instance_id(),
                                                   uuid,
                                                   sender)).unwrap();
        let service = receiver.recv().unwrap();
        match service {
            Ok(service) => {
                Ok(BluetoothRemoteGATTService::new(&self.global(),
                                                   &self.device.get(),
                                                   DOMString::from(service.uuid),
                                                   service.is_primary,
                                                   service.instance_id))
            },
            Err(error) => {
                Err(Error::from(error))
            },
        }
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-getincludedservices
    fn get_included_services(&self,
                             service: Option<BluetoothServiceUUID>)
                             -> Fallible<Vec<Root<BluetoothRemoteGATTService>>> {
        let mut uuid: Option<String> = None;
        if let Some(s) = service {
            uuid = Some(try!(BluetoothUUID::service(s)).to_string());
            if let Some(ref uuid) = uuid {
                if uuid_is_blacklisted(uuid.as_ref(), Blacklist::All) {
                    return Err(Security)
                }
            }
        };
        if !self.Device().Gatt().Connected() {
            return Err(Network)
        }
        let (sender, receiver) = ipc::channel().unwrap();
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::GetIncludedServices(self.get_instance_id(),
                                                    uuid,
                                                    sender)).unwrap();
        let services_vec = receiver.recv().unwrap();
        match services_vec {
            Ok(service_vec) => {
                Ok(service_vec.into_iter()
                              .map(|service| BluetoothRemoteGATTService::new(&self.global(),
                                                                             &self.device.get(),
                                                                             DOMString::from(service.uuid),
                                                                             service.is_primary,
                                                                             service.instance_id))
                              .collect())
            },
            Err(error) => {
                Err(Error::from(error))
            },
        }
    }
}

impl BluetoothRemoteGATTServiceMethods for BluetoothRemoteGATTService {
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-device
    fn Device(&self) -> Root<BluetoothDevice> {
        self.device.get()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-isprimary
    fn IsPrimary(&self) -> bool {
        self.is_primary
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-uuid
    fn Uuid(&self) -> DOMString {
        self.uuid.clone()
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-getcharacteristic
    fn GetCharacteristic(&self,
                         characteristic: BluetoothCharacteristicUUID)
                         -> Rc<Promise> {
        result_to_promise(&self.global(), self.get_characteristic(characteristic))
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-getcharacteristics
    fn GetCharacteristics(&self,
                          characteristic: Option<BluetoothCharacteristicUUID>)
                          -> Rc<Promise> {
        result_to_promise(&self.global(), self.get_characteristics(characteristic))
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-getincludedservice
    fn GetIncludedService(&self,
                          service: BluetoothServiceUUID)
                          -> Rc<Promise> {
        result_to_promise(&self.global(), self.get_included_service(service))
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-getincludedservices
    fn GetIncludedServices(&self,
                          service: Option<BluetoothServiceUUID>)
                          -> Rc<Promise> {
        result_to_promise(&self.global(), self.get_included_services(service))
    }
}
