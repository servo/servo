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
use dom::bindings::refcounted::{Trusted, TrustedPromise};
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::bluetoothdevice::BluetoothDevice;
use dom::bluetoothremotegattservice::BluetoothRemoteGATTService;
use dom::bluetoothuuid::{BluetoothServiceUUID, BluetoothUUID};
use dom::promise::Promise;
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use js::jsapi::JSAutoCompartment;
use net_traits::bluetooth_thread::{BluetoothMethodMsg, BluetoothResponseListener, BluetoothResultMsg};
use network_listener::{NetworkListener, PreInvoke};
use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

struct BluetoothServerContext {
    promise: Option<TrustedPromise>,
    server: Trusted<BluetoothRemoteGATTServer>,
}

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

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-connect
    fn Connect(&self) -> Rc<Promise> {
        let p = Promise::new(self.global().r());
        let (sender, receiver) = ipc::channel().unwrap();
        let bts_context = Arc::new(Mutex::new(BluetoothServerContext {
            promise: Some(TrustedPromise::new(p.clone())),
            server: Trusted::new(self),
        }));
        let listener = NetworkListener {
            context: bts_context,
            script_chan: self.global().r().networking_task_source(),
            wrapper: None,
        };
        ROUTER.add_route(receiver.to_opaque(), box move |message| {
            listener.notify_response(message.to().unwrap());
        });
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::GATTServerConnect(String::from(self.Device().Id()), sender)).unwrap();
        return p;
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
        let (sender, receiver) = ipc::channel().unwrap();
        let bts_context = Arc::new(Mutex::new(BluetoothServerContext {
            promise: Some(TrustedPromise::new(p.clone())),
            server: Trusted::new(self),
        }));
        let listener = NetworkListener {
            context: bts_context,
            script_chan: self.global().r().networking_task_source(),
            wrapper: None,
        };
        ROUTER.add_route(receiver.to_opaque(), box move |message| {
            listener.notify_response(message.to().unwrap());
        });
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::GetPrimaryService(String::from(self.Device().Id()), uuid, sender)).unwrap();
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
        let (sender, receiver) = ipc::channel().unwrap();
        let bts_context = Arc::new(Mutex::new(BluetoothServerContext {
            promise: Some(TrustedPromise::new(p.clone())),
            server: Trusted::new(self),
        }));
        let listener = NetworkListener {
            context: bts_context,
            script_chan: self.global().r().networking_task_source(),
            wrapper: None,
        };
        ROUTER.add_route(receiver.to_opaque(), box move |message| {
            listener.notify_response(message.to().unwrap());
        });
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::GetPrimaryServices(String::from(self.Device().Id()), uuid, sender)).unwrap();
        return p;
    }
}

impl PreInvoke for BluetoothServerContext {}

impl BluetoothResponseListener for BluetoothServerContext {
    #[allow(unrooted_must_root)]
    fn response(&mut self, result: BluetoothResultMsg) {
        let promise = self.promise.take().expect("bt promise is missing").root();
        let promise_cx = promise.global().r().get_cx();

        // JSAutoCompartment needs to be manually made.
        // Otherwise, Servo will crash.
        let _ac = JSAutoCompartment::new(promise_cx, promise.reflector().get_jsobject().get());
        match result {
            BluetoothResultMsg::GATTServerConnect(connected) => {
                self.server.root().connected.set(connected);
                promise.resolve_native(
                    promise_cx,
                    &self.server.root());
            },
            BluetoothResultMsg::GetPrimaryService(service) => {
                let s = BluetoothRemoteGATTService::new(self.server.root().global().r(),
                                                        &self.server.root().device.get(),
                                                        DOMString::from(service.uuid),
                                                        service.is_primary,
                                                        service.instance_id);
                promise.resolve_native(
                    promise_cx,
                    &s);
            },
            BluetoothResultMsg::GetPrimaryServices(services_vec) => {
                let s: Vec<Root<BluetoothRemoteGATTService>> =
                    services_vec.into_iter()
                                .map(|service| BluetoothRemoteGATTService::new(self.server.root().global().r(),
                                                                               &self.server.root().device.get(),
                                                                               DOMString::from(service.uuid),
                                                                               service.is_primary,
                                                                               service.instance_id))
                               .collect();
                promise.resolve_native(
                    promise_cx,
                    &s);
            },
            BluetoothResultMsg::Error(error) => {
                promise.reject_error(
                    promise_cx,
                    Error::from(error));
            },
            _ => {
                promise.reject_error(
                    promise_cx,
                    Error::Type("Something went wrong...".to_owned()));
            }
        }
        self.promise = Some(TrustedPromise::new(promise));
    }
}
