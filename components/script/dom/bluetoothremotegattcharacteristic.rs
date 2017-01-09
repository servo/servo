/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bluetooth_traits::{BluetoothRequest, BluetoothResponse, GATTType};
use bluetooth_traits::blocklist::{Blocklist, uuid_is_blocklisted};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BluetoothCharacteristicPropertiesBinding::
    BluetoothCharacteristicPropertiesMethods;
use dom::bindings::codegen::Bindings::BluetoothDeviceBinding::BluetoothDeviceMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTCharacteristicBinding;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTCharacteristicBinding::
    BluetoothRemoteGATTCharacteristicMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServerBinding::BluetoothRemoteGATTServerMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServiceBinding::BluetoothRemoteGATTServiceMethods;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::error::Error::{self, InvalidModification, Network, NotSupported, Security};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::str::{ByteString, DOMString};
use dom::bluetooth::{AsyncBluetoothListener, get_gatt_children, response_async};
use dom::bluetoothcharacteristicproperties::BluetoothCharacteristicProperties;
use dom::bluetoothremotegattservice::BluetoothRemoteGATTService;
use dom::bluetoothuuid::{BluetoothDescriptorUUID, BluetoothUUID};
use dom::eventtarget::EventTarget;
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
    eventtarget: EventTarget,
    service: JS<BluetoothRemoteGATTService>,
    uuid: DOMString,
    properties: JS<BluetoothCharacteristicProperties>,
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
            eventtarget: EventTarget::new_inherited(),
            service: JS::from_ref(service),
            uuid: uuid,
            properties: JS::from_ref(properties),
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
        Root::from_ref(&self.properties)
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-service
    fn Service(&self) -> Root<BluetoothRemoteGATTService> {
        Root::from_ref(&self.service)
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-uuid
    fn Uuid(&self) -> DOMString {
        self.uuid.clone()
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-getdescriptor
    fn GetDescriptor(&self, descriptor: BluetoothDescriptorUUID) -> Rc<Promise> {
        get_gatt_children(self, true, BluetoothUUID::descriptor, Some(descriptor), self.get_instance_id(),
                          self.Service().Device().Gatt().Connected(), GATTType::Descriptor)
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-getdescriptors
    fn GetDescriptors(&self,
                      descriptor: Option<BluetoothDescriptorUUID>)
                      -> Rc<Promise> {
        get_gatt_children(self, false, BluetoothUUID::descriptor, descriptor, self.get_instance_id(),
                          self.Service().Device().Gatt().Connected(), GATTType::Descriptor)
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

        // Step 1.
        if uuid_is_blocklisted(self.uuid.as_ref(), Blocklist::Reads) {
            p.reject_error(p_cx, Security);
            return p;
        }

        // Step 2.
        if !self.Service().Device().Gatt().Connected() {
            p.reject_error(p_cx, Network);
            return p;
        }

        // TODO: Step 5: Implement the `connection-checking-wrapper` algorithm for BluetoothRemoteGATTServer.

        // Step 5.1.
        if !self.Properties().Read() {
            p.reject_error(p_cx, NotSupported);
            return p;
        }

        // Note: Steps 3 - 4 and the remaining substeps of Step 5 are implemented in components/bluetooth/lib.rs
        // in readValue function and in handle_response function.
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

        // Step 1.
        if uuid_is_blocklisted(self.uuid.as_ref(), Blocklist::Writes) {
            p.reject_error(p_cx, Security);
            return p;
        }

        // Step 2 - 3.
        if value.len() > MAXIMUM_ATTRIBUTE_LENGTH {
            p.reject_error(p_cx, InvalidModification);
            return p;
        }

        // Step 4.
        if !self.Service().Device().Gatt().Connected() {
            p.reject_error(p_cx, Network);
            return p;
        }

        // TODO: Step 7: Implement the `connection-checking-wrapper` algorithm for BluetoothRemoteGATTServer.

        // Step 7.1.
        if !(self.Properties().Write() ||
             self.Properties().WriteWithoutResponse() ||
             self.Properties().AuthenticatedSignedWrites()) {
            p.reject_error(p_cx, NotSupported);
            return p;
        }

        // Note: Steps 5 - 6 and the remaining substeps of Step 7 are implemented in components/bluetooth/lib.rs
        // in writeValue function and in handle_response function.
        let sender = response_async(&p, self);
        self.get_bluetooth_thread().send(
            BluetoothRequest::WriteValue(self.get_instance_id(), value, sender)).unwrap();
        return p;
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-startnotifications
    fn StartNotifications(&self) -> Rc<Promise> {
        let p = Promise::new(&self.global());
        let p_cx = p.global().get_cx();

        // Step 1.
        if uuid_is_blocklisted(self.uuid.as_ref(), Blocklist::Reads) {
            p.reject_error(p_cx, Security);
            return p;
        }

        // Step 4.
        if !(self.Properties().Notify() ||
             self.Properties().Indicate()) {
            p.reject_error(p_cx, NotSupported);
            return p;
        }

        // TODO: Step 5: Implement `active notification context set` for BluetoothRemoteGATTCharacteristic.

        // Step 6.
        if !self.Service().Device().Gatt().Connected() {
            p.reject_error(p_cx, Network);
            return p;
        }

        // Note: Steps 2 - 3, 7 - 11 are implemented in components/bluetooth/lib.rs in enable_notification function
        // and in handle_response function.
        let sender = response_async(&p, self);
        self.get_bluetooth_thread().send(
            BluetoothRequest::EnableNotification(self.get_instance_id(),
                                                 true,
                                                 sender)).unwrap();
        return p;
    }

    #[allow(unrooted_must_root)]
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-stopnotifications
    fn StopNotifications(&self) -> Rc<Promise> {
        let p = Promise::new(&self.global());
        let sender = response_async(&p, self);

        // TODO: Step 3 - 4: Implement `active notification context set` for BluetoothRemoteGATTCharacteristic,

        // Note: Steps 1 - 2, and part of Step 4 and Step 5 are implemented in components/bluetooth/lib.rs
        // in enable_notification function and in handle_response function.
        self.get_bluetooth_thread().send(
            BluetoothRequest::EnableNotification(self.get_instance_id(),
                                                 false,
                                                 sender)).unwrap();
        return p;
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-characteristiceventhandlers-oncharacteristicvaluechanged
    event_handler!(characteristicvaluechanged, GetOncharacteristicvaluechanged, SetOncharacteristicvaluechanged);
}

impl AsyncBluetoothListener for BluetoothRemoteGATTCharacteristic {
    fn handle_response(&self, response: BluetoothResponse, promise_cx: *mut JSContext, promise: &Rc<Promise>) {
        let device = self.Service().Device();
        match response {
            // https://webbluetoothcg.github.io/web-bluetooth/#getgattchildren
            // Step 7.
            BluetoothResponse::GetDescriptors(descriptors_vec, single) => {
                if single {
                    promise.resolve_native(promise_cx, &device.get_or_create_descriptor(&descriptors_vec[0], &self));
                    return;
                }
                let mut descriptors = vec!();
                for descriptor in descriptors_vec {
                    let bt_descriptor = device.get_or_create_descriptor(&descriptor, &self);
                    descriptors.push(bt_descriptor);
                }
                promise.resolve_native(promise_cx, &descriptors);
            },
            // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-readvalue
            BluetoothResponse::ReadValue(result) => {
                // TODO: Step 5.5.1: Implement activeAlgorithms internal slot for BluetoothRemoteGATTServer.

                // Step 5.5.2.
                // TODO(#5014): Replace ByteString with ArrayBuffer when it is implemented.
                let value = ByteString::new(result);
                *self.value.borrow_mut() = Some(value.clone());

                // Step 5.5.3.
                self.upcast::<EventTarget>().fire_bubbling_event(atom!("characteristicvaluechanged"));

                // Step 5.5.4.
                promise.resolve_native(promise_cx, &value);
            },
            // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-writevalue
            BluetoothResponse::WriteValue(result) => {
                // TODO: Step 7.5.1: Implement activeAlgorithms internal slot for BluetoothRemoteGATTServer.

                // Step 7.5.2.
                // TODO(#5014): Replace ByteString with an ArrayBuffer wrapped in a DataView.
                *self.value.borrow_mut() = Some(ByteString::new(result));

                // Step 7.5.3.
                promise.resolve_native(promise_cx, &());
            },
            // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-startnotifications
            // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-stopnotifications
            BluetoothResponse::EnableNotification(_result) => {
                // (StartNotification) TODO: Step 10:  Implement `active notification context set`
                // for BluetoothRemoteGATTCharacteristic.

                // (StartNotification) Step 11.
                // (StopNotification)  Step 5.
                promise.resolve_native(promise_cx, self);
            },
            _ => promise.reject_error(promise_cx, Error::Type("Something went wrong...".to_owned())),
        }
    }
}
