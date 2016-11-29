/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bluetooth_traits::{BluetoothRequest, BluetoothResponse};
use bluetooth_traits::blocklist::{Blocklist, uuid_is_blocklisted};
use dom::bindings::codegen::Bindings::BluetoothDeviceBinding::BluetoothDeviceMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServerBinding::BluetoothRemoteGATTServerMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServiceBinding;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServiceBinding::BluetoothRemoteGATTServiceMethods;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::error::Error::{self, Network, Security};
use dom::bindings::js::{JS, MutHeap, Root};
use dom::bindings::reflector::{Reflectable, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::bluetooth::{AsyncBluetoothListener, response_async};
use dom::bluetoothcharacteristicproperties::BluetoothCharacteristicProperties;
use dom::bluetoothdevice::BluetoothDevice;
use dom::bluetoothremotegattcharacteristic::BluetoothRemoteGATTCharacteristic;
use dom::bluetoothuuid::{BluetoothCharacteristicUUID, BluetoothServiceUUID, BluetoothUUID};
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;
use dom::promise::Promise;
use ipc_channel::ipc::IpcSender;
use js::jsapi::JSContext;
use std::rc::Rc;

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattservice
#[dom_struct]
pub struct BluetoothRemoteGATTService {
    eventtarget: EventTarget,
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
            eventtarget: EventTarget::new_inherited(),
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

    pub fn get_device(&self) -> Root<BluetoothDevice> {
        self.device.get()
    }

    fn get_bluetooth_thread(&self) -> IpcSender<BluetoothRequest> {
        self.global().as_window().bluetooth_thread()
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
        let p = Promise::new(&self.global());
        let p_cx = p.global().get_cx();

        // Step 1.
        let uuid = match BluetoothUUID::characteristic(characteristic) {
            Ok(uuid) => uuid.to_string(),
            Err(e) => {
                p.reject_error(p_cx, e);
                return p;
            }
        };

        // Step 2.
        if uuid_is_blocklisted(uuid.as_ref(), Blocklist::All) {
            p.reject_error(p_cx, Security);
            return p;
        }

        // Step 3 - 4.
        if !self.Device().Gatt().Connected() {
            p.reject_error(p_cx, Network);
            return p;
        }

        // TODO: Step 5: Implement representedService internal slot for BluetootRemoteGATTService.

        // Note: Steps 6 - 7 are implemented is components/bluetooth/lib.rs in get_characteristic function
        // and in handle_response function.
        let sender = response_async(&p, self);
        self.get_bluetooth_thread().send(
            BluetoothRequest::GetCharacteristic(self.get_instance_id(), uuid, sender)).unwrap();
        return p;
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-getcharacteristics
    fn GetCharacteristics(&self,
                          characteristic: Option<BluetoothCharacteristicUUID>)
                          -> Rc<Promise> {
        let p = Promise::new(&self.global());
        let p_cx = p.global().get_cx();
        let mut uuid: Option<String> = None;
        if let Some(c) = characteristic {
            // Step 1.
            uuid = match BluetoothUUID::characteristic(c) {
                Ok(uuid) => Some(uuid.to_string()),
                Err(e) => {
                    p.reject_error(p_cx, e);
                    return p;
                }
            };
            if let Some(ref uuid) = uuid {
                // Step 2.
                if uuid_is_blocklisted(uuid.as_ref(), Blocklist::All) {
                    p.reject_error(p_cx, Security);
                    return p;
                }
            }
        };

        // Step 3 - 4.
        if !self.Device().Gatt().Connected() {
            p.reject_error(p_cx, Network);
            return p;
        }

        // TODO: Step 5: Implement representedService internal slot for BluetootRemoteGATTService.

        // Note: Steps 6 - 7 are implemented is components/bluetooth/lib.rs in get_characteristics function
        // and in handle_response function.
        let sender = response_async(&p, self);
        self.get_bluetooth_thread().send(
            BluetoothRequest::GetCharacteristics(self.get_instance_id(), uuid, sender)).unwrap();
        return p;
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-getincludedservice
    fn GetIncludedService(&self,
                          service: BluetoothServiceUUID)
                          -> Rc<Promise> {
        let p = Promise::new(&self.global());
        let p_cx = p.global().get_cx();

        // Step 1.
        let uuid = match BluetoothUUID::service(service) {
            Ok(uuid) => uuid.to_string(),
            Err(e) => {
                p.reject_error(p_cx, e);
                return p;
            }
        };

        // Step 2.
        if uuid_is_blocklisted(uuid.as_ref(), Blocklist::All) {
            p.reject_error(p_cx, Security);
            return p;
        }

        // Step 3 - 4.
        if !self.Device().Gatt().Connected() {
            p.reject_error(p_cx, Network);
            return p;
        }

        // TODO: Step 5: Implement representedService internal slot for BluetootRemoteGATTService.

        // Note: Steps 6 - 7 are implemented is components/bluetooth/lib.rs in get_included_service function
        // and in handle_response function.
        let sender = response_async(&p, self);
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
        let p = Promise::new(&self.global());
        let p_cx = p.global().get_cx();
        let mut uuid: Option<String> = None;
        if let Some(s) = service {
            // Step 1.
            uuid = match BluetoothUUID::service(s) {
                Ok(uuid) => Some(uuid.to_string()),
                Err(e) => {
                    p.reject_error(p_cx, e);
                    return p;
                }
            };
            if let Some(ref uuid) = uuid {
                // Step 2.
                if uuid_is_blocklisted(uuid.as_ref(), Blocklist::All) {
                    p.reject_error(p_cx, Security);
                    return p;
                }
            }
        };

        // Step 3 - 4.
        if !self.Device().Gatt().Connected() {
            p.reject_error(p_cx, Network);
            return p;
        }

        // TODO: Step 5: Implement representedService internal slot for BluetootRemoteGATTService.

        // Note: Steps 6 - 7 are implemented is components/bluetooth/lib.rs in get_included_services function
        // and in handle_response function.
        let sender = response_async(&p, self);
        self.get_bluetooth_thread().send(
            BluetoothRequest::GetIncludedServices(self.get_instance_id(),
                                                    uuid,
                                                    sender)).unwrap();
        return p;
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-serviceeventhandlers-onserviceadded
    event_handler!(serviceadded, GetOnserviceadded, SetOnserviceadded);

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-serviceeventhandlers-onservicechanged
    event_handler!(servicechanged, GetOnservicechanged, SetOnservicechanged);

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-serviceeventhandlers-onserviceremoved
    event_handler!(serviceremoved, GetOnserviceremoved, SetOnserviceremoved);
}

impl AsyncBluetoothListener for BluetoothRemoteGATTService {
    fn handle_response(&self, response: BluetoothResponse, promise_cx: *mut JSContext, promise: &Rc<Promise>) {
        match response {
            // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-getcharacteristic
            // Step 7.
            BluetoothResponse::GetCharacteristic(characteristic) => {
                let context = self.device.get().get_context();
                let mut characteristic_map = context.get_characteristic_map().borrow_mut();
                if let Some(existing_characteristic) = characteristic_map.get(&characteristic.instance_id) {
                    return promise.resolve_native(promise_cx, &existing_characteristic.get());
                }
                let properties =
                    BluetoothCharacteristicProperties::new(&self.global(),
                                                           characteristic.broadcast,
                                                           characteristic.read,
                                                           characteristic.write_without_response,
                                                           characteristic.write,
                                                           characteristic.notify,
                                                           characteristic.indicate,
                                                           characteristic.authenticated_signed_writes,
                                                           characteristic.reliable_write,
                                                           characteristic.writable_auxiliaries);
                let bt_characteristic = BluetoothRemoteGATTCharacteristic::new(&self.global(),
                                                                               self,
                                                                               DOMString::from(characteristic.uuid),
                                                                               &properties,
                                                                               characteristic.instance_id.clone());
                characteristic_map.insert(characteristic.instance_id, MutHeap::new(&bt_characteristic));
                promise.resolve_native(promise_cx, &bt_characteristic);
            },
            // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-getcharacteristics
            // Step 7.
            BluetoothResponse::GetCharacteristics(characteristics_vec) => {
                let mut characteristics = vec!();
                let context = self.device.get().get_context();
                let mut characteristic_map = context.get_characteristic_map().borrow_mut();
                for characteristic in characteristics_vec {
                    let bt_characteristic = match characteristic_map.get(&characteristic.instance_id) {
                        Some(existing_characteristic) => existing_characteristic.get(),
                        None => {
                            let properties =
                                BluetoothCharacteristicProperties::new(&self.global(),
                                                                       characteristic.broadcast,
                                                                       characteristic.read,
                                                                       characteristic.write_without_response,
                                                                       characteristic.write,
                                                                       characteristic.notify,
                                                                       characteristic.indicate,
                                                                       characteristic.authenticated_signed_writes,
                                                                       characteristic.reliable_write,
                                                                       characteristic.writable_auxiliaries);

                            BluetoothRemoteGATTCharacteristic::new(&self.global(),
                                                                   self,
                                                                   DOMString::from(characteristic.uuid),
                                                                   &properties,
                                                                   characteristic.instance_id.clone())
                        },
                    };
                    if !characteristic_map.contains_key(&characteristic.instance_id) {
                        characteristic_map.insert(characteristic.instance_id, MutHeap::new(&bt_characteristic));
                    }
                    characteristics.push(bt_characteristic);
                }
                promise.resolve_native(promise_cx, &characteristics);
            },
            // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-getincludedservice
            // Step 7.
            BluetoothResponse::GetIncludedService(service) => {
                let s =
                    BluetoothRemoteGATTService::new(&self.global(),
                                                    &self.device.get(),
                                                    DOMString::from(service.uuid),
                                                    service.is_primary,
                                                    service.instance_id);
                promise.resolve_native(promise_cx, &s);
            },
            // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattservice-getincludedservices
            // Step 7.
            BluetoothResponse::GetIncludedServices(services_vec) => {
                let s: Vec<Root<BluetoothRemoteGATTService>> =
                    services_vec.into_iter()
                                .map(|service| BluetoothRemoteGATTService::new(&self.global(),
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
