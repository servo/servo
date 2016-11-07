/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bluetooth_traits::BluetoothMethodMsg;
use bluetooth_traits::blacklist::{Blacklist, uuid_is_blacklisted};
use dom::bindings::codegen::Bindings::BluetoothDeviceBinding::BluetoothDeviceMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServerBinding;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServerBinding::BluetoothRemoteGATTServerMethods;
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::error::Error::{self, Network, Security};
use dom::bindings::js::{JS, MutHeap, Root};
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::bluetooth::result_to_promise;
use dom::bluetoothdevice::BluetoothDevice;
use dom::bluetoothremotegattservice::BluetoothRemoteGATTService;
use dom::bluetoothuuid::{BluetoothServiceUUID, BluetoothUUID};
use dom::globalscope::GlobalScope;
use dom::promise::Promise;
use ipc_channel::ipc::{self, IpcSender};
use std::cell::Cell;
use std::rc::Rc;

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattserver
#[dom_struct]
pub struct BluetoothRemoteGATTServer {
    reflector_: Reflector,
    device: MutHeap<JS<BluetoothDevice>>,
    connected: Cell<bool>,
}

impl BluetoothRemoteGATTServer {
    pub fn new_inherited(device: &BluetoothDevice) -> BluetoothRemoteGATTServer {
        BluetoothRemoteGATTServer {
            reflector_: Reflector::new(),
            device: MutHeap::new(device),
            connected: Cell::new(false),
        }
    }

    pub fn new(global: &GlobalScope, device: &BluetoothDevice) -> Root<BluetoothRemoteGATTServer> {
        reflect_dom_object(box BluetoothRemoteGATTServer::new_inherited(device),
                           global,
                           BluetoothRemoteGATTServerBinding::Wrap)
    }

    fn get_bluetooth_thread(&self) -> IpcSender<BluetoothMethodMsg> {
        self.global().as_window().bluetooth_thread()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-connect
    fn connect(&self) -> Fallible<Root<BluetoothRemoteGATTServer>> {
        let (sender, receiver) = ipc::channel().unwrap();
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::GATTServerConnect(String::from(self.Device().Id()), sender)).unwrap();
        let server = receiver.recv().unwrap();
        match server {
            Ok(connected) => {
                self.connected.set(connected);
                Ok(Root::from_ref(self))
            },
            Err(error) => {
                Err(Error::from(error))
            },
        }
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-getprimaryservice
    fn get_primary_service(&self, service: BluetoothServiceUUID) -> Fallible<Root<BluetoothRemoteGATTService>> {
        let uuid = try!(BluetoothUUID::service(service)).to_string();
        if uuid_is_blacklisted(uuid.as_ref(), Blacklist::All) {
            return Err(Security)
        }
        if !self.Device().Gatt().Connected() {
            return Err(Network)
        }
        let (sender, receiver) = ipc::channel().unwrap();
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::GetPrimaryService(String::from(self.Device().Id()), uuid, sender)).unwrap();
        let service = receiver.recv().unwrap();
        match service {
            Ok(service) => {
                let context = self.device.get().get_context();
                let mut service_map = context.get_service_map().borrow_mut();
                if let Some(existing_service) = service_map.get(&service.instance_id) {
                    return Ok(existing_service.get());
                }
                let bt_service = BluetoothRemoteGATTService::new(&self.global(),
                                                                 &self.device.get(),
                                                                 DOMString::from(service.uuid),
                                                                 service.is_primary,
                                                                 service.instance_id.clone());
                service_map.insert(service.instance_id, MutHeap::new(&bt_service));
                Ok(bt_service)
            },
            Err(error) => {
                Err(Error::from(error))
            },
        }
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-getprimaryservices
    fn get_primary_services(&self,
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
        let mut services = vec!();
        let (sender, receiver) = ipc::channel().unwrap();
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::GetPrimaryServices(String::from(self.Device().Id()), uuid, sender)).unwrap();
        let services_vec = receiver.recv().unwrap();
        match services_vec {
            Ok(service_vec) => {
                let context = self.device.get().get_context();
                let mut service_map = context.get_service_map().borrow_mut();
                for service in service_vec {
                    let bt_service = match service_map.get(&service.instance_id) {
                        Some(existing_service) => existing_service.get(),
                        None => {
                            BluetoothRemoteGATTService::new(&self.global(),
                                                            &self.device.get(),
                                                            DOMString::from(service.uuid),
                                                            service.is_primary,
                                                            service.instance_id.clone())
                        },
                    };
                    if !service_map.contains_key(&service.instance_id) {
                        service_map.insert(service.instance_id, MutHeap::new(&bt_service));
                    }
                    services.push(bt_service);
                }
                Ok(services)
            },
            Err(error) => {
                Err(Error::from(error))
            },
        }
    }
}

impl BluetoothRemoteGATTServerMethods for BluetoothRemoteGATTServer {
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-device
    fn Device(&self) -> Root<BluetoothDevice> {
        self.device.get()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-connected
    fn Connected(&self) -> bool {
        self.connected.get()
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-connect
    fn Connect(&self) -> Rc<Promise> {
        result_to_promise(&self.global(), self.connect())
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-disconnect
    fn Disconnect(&self) -> ErrorResult {
        let (sender, receiver) = ipc::channel().unwrap();
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::GATTServerDisconnect(String::from(self.Device().Id()), sender)).unwrap();
        let server = receiver.recv().unwrap();
        match server {
            Ok(connected) => {
                self.connected.set(connected);
                Ok(())
            },
            Err(error) => {
                Err(Error::from(error))
            },
        }
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-getprimaryservice
    fn GetPrimaryService(&self, service: BluetoothServiceUUID) -> Rc<Promise> {
        result_to_promise(&self.global(), self.get_primary_service(service))
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-getprimaryservices
    fn GetPrimaryServices(&self,
                          service: Option<BluetoothServiceUUID>)
                          -> Rc<Promise> {
        result_to_promise(&self.global(), self.get_primary_services(service))
    }
}
