/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bluetooth_blacklist::{Blacklist, uuid_is_blacklisted};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BluetoothCharacteristicPropertiesBinding::
    BluetoothCharacteristicPropertiesMethods;
use dom::bindings::codegen::Bindings::BluetoothDeviceBinding::BluetoothDeviceMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTCharacteristicBinding;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTCharacteristicBinding::
    BluetoothRemoteGATTCharacteristicMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServerBinding::BluetoothRemoteGATTServerMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServiceBinding::BluetoothRemoteGATTServiceMethods;
use dom::bindings::error::Error::{self, InvalidModification, Network, NotSupported, Security};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, MutHeap, Root};
use dom::bindings::refcounted::{Trusted, TrustedPromise};
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::str::{ByteString, DOMString};
use dom::bluetoothcharacteristicproperties::BluetoothCharacteristicProperties;
use dom::bluetoothremotegattdescriptor::BluetoothRemoteGATTDescriptor;
use dom::bluetoothremotegattservice::BluetoothRemoteGATTService;
use dom::bluetoothuuid::{BluetoothDescriptorUUID, BluetoothUUID};
use dom::promise::Promise;
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use js::jsapi::JSAutoCompartment;
use net_traits::bluetooth_thread::{BluetoothMethodMsg, BluetoothResponseListener, BluetoothResultMsg};
use network_listener::{NetworkListener, PreInvoke};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

struct BluetoothCharacteristicContext {
    promise: Option<TrustedPromise>,
    characteristic: Trusted<BluetoothRemoteGATTCharacteristic>,
}

// Maximum length of an attribute value.
// https://www.bluetooth.org/DocMan/handlers/DownloadDoc.ashx?doc_id=286439 (Vol. 3, page 2169)
pub const MAXIMUM_ATTRIBUTE_LENGTH: usize = 512;

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattcharacteristic
#[dom_struct]
pub struct BluetoothRemoteGATTCharacteristic {
    reflector_: Reflector,
    service: MutHeap<JS<BluetoothRemoteGATTService>>,
    uuid: DOMString,
    properties: MutHeap<JS<BluetoothCharacteristicProperties>>,
    value: DOMRefCell<Option<ByteString>>,
    instance_id: String,
}

impl BluetoothRemoteGATTCharacteristic {
    pub fn new_inherited(service: &BluetoothRemoteGATTService,
                         uuid: DOMString,
                         properties: &BluetoothCharacteristicProperties,
                         instance_id: String)
                         -> BluetoothRemoteGATTCharacteristic {
        BluetoothRemoteGATTCharacteristic {
            reflector_: Reflector::new(),
            service: MutHeap::new(service),
            uuid: uuid,
            properties: MutHeap::new(properties),
            value: DOMRefCell::new(None),
            instance_id: instance_id,
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

    fn get_instance_id(&self) -> String {
        self.instance_id.clone()
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

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-getdescriptor
    fn GetDescriptor(&self, descriptor: BluetoothDescriptorUUID) -> Rc<Promise> {
        let p = Promise::new(self.global().r());
        let p_cx = p.global().r().get_cx();
        let uuid = match BluetoothUUID::descriptor(descriptor) {
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
        let btc_context = Arc::new(Mutex::new(BluetoothCharacteristicContext {
            promise: Some(TrustedPromise::new(p.clone())),
            characteristic: Trusted::new(self),
        }));
        let listener = NetworkListener {
            context: btc_context,
            script_chan: self.global().r().networking_task_source(),
            wrapper: None,
        };
        ROUTER.add_route(receiver.to_opaque(), box move |message| {
            listener.notify_response(message.to().unwrap());
        });
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::GetDescriptor(self.get_instance_id(), uuid, sender)).unwrap();
        return p;
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-getdescriptors
    fn GetDescriptors(&self,
                      descriptor: Option<BluetoothDescriptorUUID>)
                      -> Rc<Promise> {
        let p = Promise::new(self.global().r());
        let p_cx = p.global().r().get_cx();
        let mut uuid: Option<String> = None;
        if let Some(d) = descriptor {
            uuid = match BluetoothUUID::descriptor(d) {
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
        let btc_context = Arc::new(Mutex::new(BluetoothCharacteristicContext {
            promise: Some(TrustedPromise::new(p.clone())),
            characteristic: Trusted::new(self),
        }));
        let listener = NetworkListener {
            context: btc_context,
            script_chan: self.global().r().networking_task_source(),
            wrapper: None,
        };
        ROUTER.add_route(receiver.to_opaque(), box move |message| {
            listener.notify_response(message.to().unwrap());
        });
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::GetDescriptors(self.get_instance_id(), uuid, sender)).unwrap();
        return p;
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-value
    fn GetValue(&self) -> Option<ByteString> {
        self.value.borrow().clone()
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-readvalue
    fn ReadValue(&self) -> Rc<Promise> {
        let p = Promise::new(self.global().r());
        let p_cx = p.global().r().get_cx();
        if uuid_is_blacklisted(self.uuid.as_ref(), Blacklist::Reads) {
            p.reject_error(p_cx, Security);
            return p;
        }
        if !self.Service().Device().Gatt().Connected() {
            p.reject_error(p_cx, Network);
            return p;
        }
        if !self.Properties().Read() {
            p.reject_error(p_cx, NotSupported);
            return p;
        }
        let (sender, receiver) = ipc::channel().unwrap();
        let btc_context = Arc::new(Mutex::new(BluetoothCharacteristicContext {
            promise: Some(TrustedPromise::new(p.clone())),
            characteristic: Trusted::new(self),
        }));
        let listener = NetworkListener {
            context: btc_context,
            script_chan: self.global().r().networking_task_source(),
            wrapper: None,
        };
        ROUTER.add_route(receiver.to_opaque(), box move |message| {
            listener.notify_response(message.to().unwrap());
        });
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::ReadValue(self.get_instance_id(), sender)).unwrap();
        return p;
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-writevalue
    fn WriteValue(&self, value: Vec<u8>) -> Rc<Promise> {
        let p = Promise::new(self.global().r());
        let p_cx = p.global().r().get_cx();
        if uuid_is_blacklisted(self.uuid.as_ref(), Blacklist::Writes) {
            p.reject_error(p_cx, Security);
            return p;
        }
        if value.len() > MAXIMUM_ATTRIBUTE_LENGTH {
            p.reject_error(p_cx, InvalidModification);
            return p;
        }
        if !self.Service().Device().Gatt().Connected() {
            p.reject_error(p_cx, Network);
            return p;
        }

        if !(self.Properties().Write() ||
             self.Properties().WriteWithoutResponse() ||
             self.Properties().AuthenticatedSignedWrites()) {
            p.reject_error(p_cx, NotSupported);
            return p;
        }
        let (sender, receiver) = ipc::channel().unwrap();
        let btc_context = Arc::new(Mutex::new(BluetoothCharacteristicContext {
            promise: Some(TrustedPromise::new(p.clone())),
            characteristic: Trusted::new(self),
        }));
        let listener = NetworkListener {
            context: btc_context,
            script_chan: self.global().r().networking_task_source(),
            wrapper: None,
        };
        ROUTER.add_route(receiver.to_opaque(), box move |message| {
            listener.notify_response(message.to().unwrap());
        });
        self.get_bluetooth_thread().send(
            BluetoothMethodMsg::WriteValue(self.get_instance_id(), value, sender)).unwrap();
        return p;
    }
}

impl PreInvoke for BluetoothCharacteristicContext {}

impl BluetoothResponseListener for BluetoothCharacteristicContext {
    #[allow(unrooted_must_root)]
    fn response(&mut self, result: BluetoothResultMsg) {
        let promise = self.promise.take().expect("bt promise is missing").root();
        let promise_cx = promise.global().r().get_cx();

        // JSAutoCompartment needs to be manually made.
        // Otherwise, Servo will crash.
        let _ac = JSAutoCompartment::new(promise_cx, promise.reflector().get_jsobject().get());
        match result {
            BluetoothResultMsg::GetDescriptor(descriptor) => {
                let d =
                    BluetoothRemoteGATTDescriptor::new(self.characteristic.root().global().r(),
                                                       self.characteristic.root().r(),
                                                       DOMString::from(descriptor.uuid),
                                                       descriptor.instance_id);
                promise.resolve_native(
                    promise_cx,
                    &d);
            },
            BluetoothResultMsg::GetDescriptors(descriptors_vec) => {
                let d: Vec<Root<BluetoothRemoteGATTDescriptor>> =
                    descriptors_vec.into_iter()
                                   .map(|desc|
                                        BluetoothRemoteGATTDescriptor::new(self.characteristic.root().global().r(),
                                                                           self.characteristic.root().r(),
                                                                           DOMString::from(desc.uuid),
                                                                           desc.instance_id))
                                  .collect();
                promise.resolve_native(
                    promise_cx,
                    &d);
            },
            BluetoothResultMsg::ReadValue(result) => {
                let value = ByteString::new(result);
                let c = self.characteristic.root();
                *c.value.borrow_mut() = Some(value.clone());
                promise.resolve_native(
                    promise_cx,
                    &value);
            },
            BluetoothResultMsg::WriteValue(result) => {
                promise.resolve_native(
                    promise_cx,
                    &result);
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
