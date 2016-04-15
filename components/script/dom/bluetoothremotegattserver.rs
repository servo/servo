/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::BluetoothDeviceBinding::BluetoothDeviceMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServerBinding;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServerBinding::BluetoothRemoteGATTServerMethods;
use dom::bindings::codegen::UnionTypes::StringOrUnsignedLong;
use dom::bindings::error::Error::Type;
use dom::bindings::error::{Fallible, ErrorResult};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, MutHeap, Root};
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bluetoothdevice::BluetoothDevice;
use dom::bluetoothremotegattservice::BluetoothRemoteGATTService;
use dom::bluetoothuuid::BluetoothUUID;
use ipc_channel::ipc::{self, IpcSender};
use net_traits::bluetooth_thread::{BluetoothMethodMsg, BluetoothObjectMsg};
use std::cell::Cell;
use util::str::DOMString;

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

    pub fn new(global: GlobalRef, device: &BluetoothDevice) -> Root<BluetoothRemoteGATTServer> {
        reflect_dom_object(box BluetoothRemoteGATTServer::new_inherited(device),
        global,
        BluetoothRemoteGATTServerBinding::Wrap)
    }

    fn get_bluetooth_thread(&self) -> IpcSender<BluetoothMethodMsg> {
        let global_root = self.global();
        let global_ref = global_root.r();
        global_ref.as_window().bluetooth_thread()
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

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-connect
    fn Connect(&self) -> Fallible<Root<BluetoothRemoteGATTServer>> {
        let (sender, receiver) = ipc::channel().unwrap();
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::GATTServerConnect(String::from(self.Device().Id()), sender)).unwrap();
        let server = receiver.recv().unwrap();
        match server {
            BluetoothObjectMsg::BluetoothServer {
                connected
            } => {
                self.connected.set(connected);
                Ok(Root::from_ref(self))
            },
            BluetoothObjectMsg::Error {
                error
            } => {
                Err(Type(error))
            },
            _ => unreachable!()
        }
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-disconnect
    fn Disconnect(&self) -> ErrorResult {
        let (sender, receiver) = ipc::channel().unwrap();
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::GATTServerDisconnect(String::from(self.Device().Id()), sender)).unwrap();
        let server = receiver.recv().unwrap();
        match server {
            BluetoothObjectMsg::BluetoothServer {
                connected
            } => {
                self.connected.set(connected);
                Ok(())
            },
            BluetoothObjectMsg::Error {
                error
            } => {
                Err(Type(error))
            },
            _ => unreachable!()
        }
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-getprimaryservice
    fn GetPrimaryService(&self, service: StringOrUnsignedLong) -> Fallible<Root<BluetoothRemoteGATTService>> {
        let uuid: String = match BluetoothUUID::GetService(self.global().r(), service.clone()) {
            Ok(domstring) => domstring.to_string(),
            Err(error) => return Err(error),
        };
        let (sender, receiver) = ipc::channel().unwrap();
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::GetPrimaryService(String::from(self.Device().Id()), uuid, sender)).unwrap();
        let service = receiver.recv().unwrap();
        match service {
            BluetoothObjectMsg::BluetoothService {
                uuid,
                is_primary,
                instance_id,
            } => {
                Ok(BluetoothRemoteGATTService::new(self.global().r(),
                                                   &self.device.get(),
                                                   DOMString::from(uuid),
                                                   is_primary,
                                                   instance_id))
            },
            BluetoothObjectMsg::Error {
                error
            } => {
                Err(Type(error))
            },
            _ => unreachable!(),
        }
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-getprimaryservices
    fn GetPrimaryServices(&self,
                          service: Option<StringOrUnsignedLong>)
                          -> Fallible<Vec<Root<BluetoothRemoteGATTService>>> {
        let mut uuid: Option<String> = None;
        if let Some(s)= service {
            match BluetoothUUID::GetService(self.global().r(), s.clone()) {
                Ok(domstring) => uuid = Some(domstring.to_string()),
                Err(error) => return Err(error),
            }
        };
        let mut services: Vec<Root<BluetoothRemoteGATTService>> = vec!();
        let (sender, receiver) = ipc::channel().unwrap();
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::GetPrimaryServices(String::from(self.Device().Id()), uuid, sender)).unwrap();
        let services_vec = receiver.recv().unwrap();
        match services_vec {
            BluetoothObjectMsg::BluetoothServices {
                services_vec
            } => {
                for s in services_vec {
                    match s {
                        BluetoothObjectMsg::BluetoothService {
                            uuid,
                            is_primary,
                            instance_id,
                        } => {
                            services.push(BluetoothRemoteGATTService::new(self.global().r(),
                                                                          &self.device.get(),
                                                                          DOMString::from(uuid),
                                                                          is_primary,
                                                                          instance_id));
                        },
                        _ => unreachable!(),
                    }
                }
                Ok(services)
            },
            BluetoothObjectMsg::Error {
                error
            } => {
                Err(Type(error))
            },
            _ => unreachable!(),
        }
    }
}
