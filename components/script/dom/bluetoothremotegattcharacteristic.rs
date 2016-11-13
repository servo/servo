/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bluetooth_traits::{BluetoothRequest, BluetoothResponse};
use bluetooth_traits::blacklist::{Blacklist, uuid_is_blacklisted};
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
use dom::bindings::js::{JS, MutHeap, Root};
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::str::{ByteString, DOMString};
use dom::bluetooth::{AsyncBluetoothListener, response_async};
use dom::bluetoothcharacteristicproperties::BluetoothCharacteristicProperties;
use dom::bluetoothremotegattdescriptor::BluetoothRemoteGATTDescriptor;
use dom::bluetoothremotegattservice::BluetoothRemoteGATTService;
use dom::bluetoothuuid::{BluetoothDescriptorUUID, BluetoothUUID};
use dom::globalscope::GlobalScope;
use dom::promise::Promise;
use ipc_channel::ipc::IpcSender;
use js::jsapi::JSContext;
use std::rc::Rc;

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

    pub fn new(global: &GlobalScope,
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

    fn get_bluetooth_thread(&self) -> IpcSender<BluetoothRequest> {
        self.global().as_window().bluetooth_thread()
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
        let p = Promise::new(&self.global());
        let p_cx = p.global().get_cx();
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
        if !self.Service().Device().Gatt().Connected() {
            p.reject_error(p_cx, Network);
            return p;
        }
        let sender = response_async(&p, self);
        self.get_bluetooth_thread().send(
            BluetoothRequest::GetDescriptor(self.get_instance_id(), uuid, sender)).unwrap();
        return p;
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-getdescriptors
    fn GetDescriptors(&self,
                      descriptor: Option<BluetoothDescriptorUUID>)
                      -> Rc<Promise> {
        let p = Promise::new(&self.global());
        let p_cx = p.global().get_cx();
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
        if !self.Service().Device().Gatt().Connected() {
            p.reject_error(p_cx, Network);
            return p;
        }
        let sender = response_async(&p, self);
        self.get_bluetooth_thread().send(
            BluetoothRequest::GetDescriptors(self.get_instance_id(), uuid, sender)).unwrap();
        return p;
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-value
    fn GetValue(&self) -> Option<ByteString> {
        self.value.borrow().clone()
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-readvalue
    fn ReadValue(&self) -> Rc<Promise> {
        let p = Promise::new(&self.global());
        let p_cx = p.global().get_cx();
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
        let sender = response_async(&p, self);
        self.get_bluetooth_thread().send(
            BluetoothRequest::ReadValue(self.get_instance_id(), sender)).unwrap();
        return p;
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-writevalue
    fn WriteValue(&self, value: Vec<u8>) -> Rc<Promise> {
        let p = Promise::new(&self.global());
        let p_cx = p.global().get_cx();
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
        let sender = response_async(&p, self);
        self.get_bluetooth_thread().send(
            BluetoothRequest::WriteValue(self.get_instance_id(), value, sender)).unwrap();
        return p;
    }
}

impl AsyncBluetoothListener for BluetoothRemoteGATTCharacteristic {
    fn handle_response(&self, response: BluetoothResponse, promise_cx: *mut JSContext, promise: &Rc<Promise>) {
        match response {
            BluetoothResponse::GetDescriptor(descriptor) => {
                let context = self.service.get().get_device().get_context();
                let mut descriptor_map = context.get_descriptor_map().borrow_mut();
                if let Some(existing_descriptor) = descriptor_map.get(&descriptor.instance_id) {
                    return promise.resolve_native(promise_cx, &existing_descriptor.get());
                }
                let bt_descriptor = BluetoothRemoteGATTDescriptor::new(&self.global(),
                                                                       self,
                                                                       DOMString::from(descriptor.uuid),
                                                                       descriptor.instance_id.clone());
                descriptor_map.insert(descriptor.instance_id, MutHeap::new(&bt_descriptor));
                promise.resolve_native(promise_cx, &bt_descriptor);
            },
            BluetoothResponse::GetDescriptors(descriptors_vec) => {
                let mut descriptors = vec!();
                let context = self.service.get().get_device().get_context();
                let mut descriptor_map = context.get_descriptor_map().borrow_mut();
                for descriptor in descriptors_vec {
                    let bt_descriptor = match descriptor_map.get(&descriptor.instance_id) {
                        Some(existing_descriptor) => existing_descriptor.get(),
                        None => {
                            BluetoothRemoteGATTDescriptor::new(&self.global(),
                                                               self,
                                                               DOMString::from(descriptor.uuid),
                                                               descriptor.instance_id.clone())
                        },
                    };
                    if !descriptor_map.contains_key(&descriptor.instance_id) {
                        descriptor_map.insert(descriptor.instance_id, MutHeap::new(&bt_descriptor));
                    }
                    descriptors.push(bt_descriptor);
                }
                promise.resolve_native(promise_cx, &descriptors);
            },
            BluetoothResponse::ReadValue(result) => {
                let value = ByteString::new(result);
                *self.value.borrow_mut() = Some(value.clone());
                promise.resolve_native(promise_cx, &value);
            },
            BluetoothResponse::WriteValue(result) => {
                let value = ByteString::new(result);
                *self.value.borrow_mut() = Some(value.clone());
                promise.resolve_native(promise_cx, &value);
            },
            _ => promise.reject_error(promise_cx, Error::Type("Something went wrong...".to_owned())),
        }
    }
}
