/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bluetooth_blacklist::{Blacklist, uuid_is_blacklisted};
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServiceBinding;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServiceBinding::BluetoothRemoteGATTServiceMethods;
use dom::bindings::error::Error::{self, Security};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, MutHeap, Root};
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::bluetooth::{AsyncBluetoothListener, response_async};
use dom::bluetoothcharacteristicproperties::BluetoothCharacteristicProperties;
use dom::bluetoothdevice::BluetoothDevice;
use dom::bluetoothremotegattcharacteristic::BluetoothRemoteGATTCharacteristic;
use dom::bluetoothuuid::{BluetoothCharacteristicUUID, BluetoothServiceUUID, BluetoothUUID};
use dom::promise::Promise;
use ipc_channel::ipc::IpcSender;
use js::jsapi::JSContext;
use net_traits::bluetooth_thread::{BluetoothRequest, BluetoothResponse};
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

    fn get_bluetooth_thread(&self) -> IpcSender<BluetoothRequest> {
        let global_root = self.global();
        let global_ref = global_root.r();
        global_ref.as_window().bluetooth_thread()
    }

    fn get_instance_id(&self) -> String {
        self.instance_id.clone()
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
       let p = Promise::new(self.global().r());
       let p_cx = p.global().r().get_cx();
       let uuid = match BluetoothUUID::characteristic(characteristic) {
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
        let sender = response_async(&p, Trusted::new(self));
        self.get_bluetooth_thread().send(
            BluetoothRequest::GetCharacteristic(self.get_instance_id(), uuid, sender)).unwrap();
        return p;
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-getcharacteristics
    fn GetCharacteristics(&self,
                          characteristic: Option<BluetoothCharacteristicUUID>)
                          -> Rc<Promise> {
        let p = Promise::new(self.global().r());
        let p_cx = p.global().r().get_cx();
        let mut uuid: Option<String> = None;
        if let Some(c) = characteristic {
            uuid = match BluetoothUUID::characteristic(c) {
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
        let sender = response_async(&p, Trusted::new(self));
        self.get_bluetooth_thread().send(
            BluetoothRequest::GetCharacteristics(self.get_instance_id(), uuid, sender)).unwrap();
        return p;
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-getincludedservice
    fn GetIncludedService(&self,
                          service: BluetoothServiceUUID)
                          -> Rc<Promise> {
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
        let sender = response_async(&p, Trusted::new(self));
        self.get_bluetooth_thread().send(
            BluetoothRequest::GetIncludedService(self.get_instance_id(),
                                                   uuid,
                                                   sender)).unwrap();
        return p;
    }


    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-getincludedservices
    fn GetIncludedServices(&self,
                          service: Option<BluetoothServiceUUID>)
                          -> Rc<Promise> {
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
        };
        let sender = response_async(&p, Trusted::new(self));
        self.get_bluetooth_thread().send(
            BluetoothRequest::GetIncludedServices(self.get_instance_id(),
                                                    uuid,
                                                    sender)).unwrap();
        return p;
    }
}

impl AsyncBluetoothListener for BluetoothRemoteGATTService {
    fn handle_response(&self, response: BluetoothResponse, promise_cx: *mut JSContext, promise: &Rc<Promise>) {
        match response {
            BluetoothResponse::GetCharacteristic(characteristic) => {
                let properties =
                BluetoothCharacteristicProperties::new(self.global().r(),
                                                       characteristic.broadcast,
                                                       characteristic.read,
                                                       characteristic.write_without_response,
                                                       characteristic.write,
                                                       characteristic.notify,
                                                       characteristic.indicate,
                                                       characteristic.authenticated_signed_writes,
                                                       characteristic.reliable_write,
                                                       characteristic.writable_auxiliaries);
                let c = BluetoothRemoteGATTCharacteristic::new(
                    self.global().r(),
                    &self,
                    DOMString::from(characteristic.uuid),
                    &properties,
                    characteristic.instance_id);
                promise.resolve_native(promise_cx, &c);
            },
            BluetoothResponse::GetCharacteristics(characteristics_vec) => {
                let mut characteristics = vec!();
                for characteristic in characteristics_vec {
                    let properties =
                        BluetoothCharacteristicProperties::new(self.global().r(),
                                                               characteristic.broadcast,
                                                               characteristic.read,
                                                               characteristic.write_without_response,
                                                               characteristic.write,
                                                               characteristic.notify,
                                                               characteristic.indicate,
                                                               characteristic.authenticated_signed_writes,
                                                               characteristic.reliable_write,
                                                               characteristic.writable_auxiliaries);
                    characteristics.push(
                        BluetoothRemoteGATTCharacteristic::new(self.global().r(),
                                                               &self,
                                                               DOMString::from(characteristic.uuid),
                                                               &properties,
                                                               characteristic.instance_id));
                }
                promise.resolve_native(promise_cx, &characteristics);
            },
            BluetoothResponse::GetIncludedService(service) => {
                let s =
                    BluetoothRemoteGATTService::new(self.global().r(),
                                                    &self.device.get(),
                                                    DOMString::from(service.uuid),
                                                    service.is_primary,
                                                    service.instance_id);
                promise.resolve_native(promise_cx, &s);
            },
            BluetoothResponse::GetIncludedServices(services_vec) => {
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
