/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bluetooth_blacklist::{Blacklist, uuid_is_blacklisted};
use dom::bindings::codegen::Bindings::BluetoothDeviceBinding::BluetoothDeviceMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServerBinding;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServerBinding::BluetoothRemoteGATTServerMethods;
use dom::bindings::error::Error::{self, Security};
use dom::bindings::error::ErrorResult;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, MutHeap, Root};
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::bluetooth::{AsyncBluetoothListener, response_async};
use dom::bluetoothdevice::BluetoothDevice;
use dom::bluetoothremotegattservice::BluetoothRemoteGATTService;
use dom::bluetoothuuid::{BluetoothServiceUUID, BluetoothUUID};
use dom::promise::Promise;
use ipc_channel::ipc::{self, IpcSender};
use js::jsapi::JSContext;
use net_traits::bluetooth_thread::{BluetoothRequest, BluetoothResponse};
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

    pub fn new(global: GlobalRef, device: &BluetoothDevice) -> Root<BluetoothRemoteGATTServer> {
        reflect_dom_object(box BluetoothRemoteGATTServer::new_inherited(device),
        global,
        BluetoothRemoteGATTServerBinding::Wrap)
    }

    fn get_bluetooth_thread(&self) -> IpcSender<BluetoothRequest> {
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

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-connect
    fn Connect(&self) -> Rc<Promise> {
        let p = Promise::new(self.global().r());
        let sender = response_async(&p, self);
        self.get_bluetooth_thread().send(
            BluetoothRequest::GATTServerConnect(String::from(self.Device().Id()), sender)).unwrap();
        return p;
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-disconnect
    fn Disconnect(&self) -> ErrorResult {
        let (sender, receiver) = ipc::channel().unwrap();
        self.get_bluetooth_thread().send(
            BluetoothRequest::GATTServerDisconnect(String::from(self.Device().Id()), sender)).unwrap();
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
        let p = Promise::new(self.global().r());
        let p_cx = p.global().r().get_cx();
        let uuid = match BluetoothUUID::service(service) {
            Ok(uuid) => uuid.to_string(),
            Err(e) => {
                p.reject_error(p_cx, e);
                return p;
            }
        };
        if uuid_is_blacklisted(uuid.as_ref(), Blacklist::All) {
            p.reject_error(p_cx, Security);
            return p;
        }
        let sender = response_async(&p, self);
        self.get_bluetooth_thread().send(
            BluetoothRequest::GetPrimaryService(String::from(self.Device().Id()), uuid, sender)).unwrap();
        return p;
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-getprimaryservices
    fn GetPrimaryServices(&self, service: Option<BluetoothServiceUUID>) -> Rc<Promise> {
        let p = Promise::new(self.global().r());
        let p_cx = p.global().r().get_cx();
        let mut uuid: Option<String> = None;
        if let Some(s) = service {
            uuid = match BluetoothUUID::service(s) {
                Ok(uuid) => Some(uuid.to_string()),
                Err(e) => {
                    p.reject_error(p_cx, e);
                    return p;
                }
            };
            if let Some(ref uuid) = uuid {
                if uuid_is_blacklisted(uuid.as_ref(), Blacklist::All) {
                    p.reject_error(p_cx, Security);
                    return p;
                }
            }
        };
        let sender = response_async(&p, self);
        self.get_bluetooth_thread().send(
            BluetoothRequest::GetPrimaryServices(String::from(self.Device().Id()), uuid, sender)).unwrap();
        return p;
    }
}

impl AsyncBluetoothListener for BluetoothRemoteGATTServer {
    fn handle_response(&self, response: BluetoothResponse, promise_cx: *mut JSContext, promise: &Rc<Promise>) {
        match response {
            BluetoothResponse::GATTServerConnect(connected) => {
                self.connected.set(connected);
                promise.resolve_native(promise_cx, self);
            },
            BluetoothResponse::GetPrimaryService(service) => {
                let s = BluetoothRemoteGATTService::new(self.global().r(),
                                                        &self.device.get(),
                                                        DOMString::from(service.uuid),
                                                        service.is_primary,
                                                        service.instance_id);
                promise.resolve_native(promise_cx, &s);
            },
            BluetoothResponse::GetPrimaryServices(services_vec) => {
                let s: Vec<Root<BluetoothRemoteGATTService>> =
                    services_vec.into_iter()
                                .map(|service| BluetoothRemoteGATTService::new(self.global().r(),
                                                                               &self.device.get(),
                                                                               DOMString::from(service.uuid),
                                                                               service.is_primary,
                                                                               service.instance_id))
                               .collect();
                promise.resolve_native(promise_cx, &s);
            },
            _ => promise.reject_error(promise_cx, Error::Type("Something went wrong...".to_owned())),
        }
    }
}
