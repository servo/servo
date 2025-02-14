/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use bluetooth_traits::blocklist::{uuid_is_blocklisted, Blocklist};
use bluetooth_traits::{BluetoothRequest, BluetoothResponse, GATTType};
use dom_struct::dom_struct;
use ipc_channel::ipc::IpcSender;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::BluetoothCharacteristicPropertiesBinding::BluetoothCharacteristicPropertiesMethods;
use crate::dom::bindings::codegen::Bindings::BluetoothRemoteGATTCharacteristicBinding::BluetoothRemoteGATTCharacteristicMethods;
use crate::dom::bindings::codegen::Bindings::BluetoothRemoteGATTServerBinding::BluetoothRemoteGATTServerMethods;
use crate::dom::bindings::codegen::Bindings::BluetoothRemoteGATTServiceBinding::BluetoothRemoteGATTServiceMethods;
use crate::dom::bindings::codegen::UnionTypes::ArrayBufferViewOrArrayBuffer;
use crate::dom::bindings::error::Error::{
    self, InvalidModification, Network, NotSupported, Security,
};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::{ByteString, DOMString};
use crate::dom::bluetooth::{get_gatt_children, response_async, AsyncBluetoothListener};
use crate::dom::bluetoothcharacteristicproperties::BluetoothCharacteristicProperties;
use crate::dom::bluetoothremotegattservice::BluetoothRemoteGATTService;
use crate::dom::bluetoothuuid::{BluetoothDescriptorUUID, BluetoothUUID};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::realms::InRealm;
use crate::script_runtime::CanGc;

// Maximum length of an attribute value.
// https://www.bluetooth.org/DocMan/handlers/DownloadDoc.ashx?doc_id=286439 (Vol. 3, page 2169)
pub(crate) const MAXIMUM_ATTRIBUTE_LENGTH: usize = 512;

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattcharacteristic
#[dom_struct]
pub(crate) struct BluetoothRemoteGATTCharacteristic {
    eventtarget: EventTarget,
    service: Dom<BluetoothRemoteGATTService>,
    uuid: DOMString,
    properties: Dom<BluetoothCharacteristicProperties>,
    value: DomRefCell<Option<ByteString>>,
    instance_id: String,
}

impl BluetoothRemoteGATTCharacteristic {
    pub(crate) fn new_inherited(
        service: &BluetoothRemoteGATTService,
        uuid: DOMString,
        properties: &BluetoothCharacteristicProperties,
        instance_id: String,
    ) -> BluetoothRemoteGATTCharacteristic {
        BluetoothRemoteGATTCharacteristic {
            eventtarget: EventTarget::new_inherited(),
            service: Dom::from_ref(service),
            uuid,
            properties: Dom::from_ref(properties),
            value: DomRefCell::new(None),
            instance_id,
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        service: &BluetoothRemoteGATTService,
        uuid: DOMString,
        properties: &BluetoothCharacteristicProperties,
        instance_id: String,
    ) -> DomRoot<BluetoothRemoteGATTCharacteristic> {
        reflect_dom_object(
            Box::new(BluetoothRemoteGATTCharacteristic::new_inherited(
                service,
                uuid,
                properties,
                instance_id,
            )),
            global,
            CanGc::note(),
        )
    }

    fn get_bluetooth_thread(&self) -> IpcSender<BluetoothRequest> {
        self.global().as_window().bluetooth_thread()
    }

    fn get_instance_id(&self) -> String {
        self.instance_id.clone()
    }
}

impl BluetoothRemoteGATTCharacteristicMethods<crate::DomTypeHolder>
    for BluetoothRemoteGATTCharacteristic
{
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-properties
    fn Properties(&self) -> DomRoot<BluetoothCharacteristicProperties> {
        DomRoot::from_ref(&self.properties)
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-service
    fn Service(&self) -> DomRoot<BluetoothRemoteGATTService> {
        DomRoot::from_ref(&self.service)
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-uuid
    fn Uuid(&self) -> DOMString {
        self.uuid.clone()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-getdescriptor
    fn GetDescriptor(&self, descriptor: BluetoothDescriptorUUID, can_gc: CanGc) -> Rc<Promise> {
        get_gatt_children(
            self,
            true,
            BluetoothUUID::descriptor,
            Some(descriptor),
            self.get_instance_id(),
            self.Service().Device().get_gatt().Connected(),
            GATTType::Descriptor,
            can_gc,
        )
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-getdescriptors
    fn GetDescriptors(
        &self,
        descriptor: Option<BluetoothDescriptorUUID>,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        get_gatt_children(
            self,
            false,
            BluetoothUUID::descriptor,
            descriptor,
            self.get_instance_id(),
            self.Service().Device().get_gatt().Connected(),
            GATTType::Descriptor,
            can_gc,
        )
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-value
    fn GetValue(&self) -> Option<ByteString> {
        self.value.borrow().clone()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-readvalue
    fn ReadValue(&self, comp: InRealm, can_gc: CanGc) -> Rc<Promise> {
        let p = Promise::new_in_current_realm(comp, can_gc);

        // Step 1.
        if uuid_is_blocklisted(self.uuid.as_ref(), Blocklist::Reads) {
            p.reject_error(Security);
            return p;
        }

        // Step 2.
        if !self.Service().Device().get_gatt().Connected() {
            p.reject_error(Network);
            return p;
        }

        // TODO: Step 5: Implement the `connection-checking-wrapper` algorithm for BluetoothRemoteGATTServer.

        // Step 5.1.
        if !self.Properties().Read() {
            p.reject_error(NotSupported);
            return p;
        }

        // Note: Steps 3 - 4 and the remaining substeps of Step 5 are implemented in components/bluetooth/lib.rs
        // in readValue function and in handle_response function.
        let sender = response_async(&p, self);
        self.get_bluetooth_thread()
            .send(BluetoothRequest::ReadValue(self.get_instance_id(), sender))
            .unwrap();
        p
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-writevalue
    fn WriteValue(
        &self,
        value: ArrayBufferViewOrArrayBuffer,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        let p = Promise::new_in_current_realm(comp, can_gc);

        // Step 1.
        if uuid_is_blocklisted(self.uuid.as_ref(), Blocklist::Writes) {
            p.reject_error(Security);
            return p;
        }

        // Step 2 - 3.
        let vec = match value {
            ArrayBufferViewOrArrayBuffer::ArrayBufferView(avb) => avb.to_vec(),
            ArrayBufferViewOrArrayBuffer::ArrayBuffer(ab) => ab.to_vec(),
        };

        if vec.len() > MAXIMUM_ATTRIBUTE_LENGTH {
            p.reject_error(InvalidModification);
            return p;
        }

        // Step 4.
        if !self.Service().Device().get_gatt().Connected() {
            p.reject_error(Network);
            return p;
        }

        // TODO: Step 7: Implement the `connection-checking-wrapper` algorithm for BluetoothRemoteGATTServer.

        // Step 7.1.
        if !(self.Properties().Write() ||
            self.Properties().WriteWithoutResponse() ||
            self.Properties().AuthenticatedSignedWrites())
        {
            p.reject_error(NotSupported);
            return p;
        }

        // Note: Steps 5 - 6 and the remaining substeps of Step 7 are implemented in components/bluetooth/lib.rs
        // in writeValue function and in handle_response function.
        let sender = response_async(&p, self);
        self.get_bluetooth_thread()
            .send(BluetoothRequest::WriteValue(
                self.get_instance_id(),
                vec,
                sender,
            ))
            .unwrap();
        p
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-startnotifications
    fn StartNotifications(&self, comp: InRealm, can_gc: CanGc) -> Rc<Promise> {
        let p = Promise::new_in_current_realm(comp, can_gc);

        // Step 1.
        if uuid_is_blocklisted(self.uuid.as_ref(), Blocklist::Reads) {
            p.reject_error(Security);
            return p;
        }

        // Step 2.
        if !self.Service().Device().get_gatt().Connected() {
            p.reject_error(Network);
            return p;
        }

        // Step 5.
        if !(self.Properties().Notify() || self.Properties().Indicate()) {
            p.reject_error(NotSupported);
            return p;
        }

        // TODO: Step 6: Implement `active notification context set` for BluetoothRemoteGATTCharacteristic.

        // Note: Steps 3 - 4, 7 - 11 are implemented in components/bluetooth/lib.rs in enable_notification function
        // and in handle_response function.
        let sender = response_async(&p, self);
        self.get_bluetooth_thread()
            .send(BluetoothRequest::EnableNotification(
                self.get_instance_id(),
                true,
                sender,
            ))
            .unwrap();
        p
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-stopnotifications
    fn StopNotifications(&self, comp: InRealm, can_gc: CanGc) -> Rc<Promise> {
        let p = Promise::new_in_current_realm(comp, can_gc);
        let sender = response_async(&p, self);

        // TODO: Step 3 - 4: Implement `active notification context set` for BluetoothRemoteGATTCharacteristic,

        // Note: Steps 1 - 2, and part of Step 4 and Step 5 are implemented in components/bluetooth/lib.rs
        // in enable_notification function and in handle_response function.
        self.get_bluetooth_thread()
            .send(BluetoothRequest::EnableNotification(
                self.get_instance_id(),
                false,
                sender,
            ))
            .unwrap();
        p
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-characteristiceventhandlers-oncharacteristicvaluechanged
    event_handler!(
        characteristicvaluechanged,
        GetOncharacteristicvaluechanged,
        SetOncharacteristicvaluechanged
    );
}

impl AsyncBluetoothListener for BluetoothRemoteGATTCharacteristic {
    fn handle_response(&self, response: BluetoothResponse, promise: &Rc<Promise>, can_gc: CanGc) {
        let device = self.Service().Device();
        match response {
            // https://webbluetoothcg.github.io/web-bluetooth/#getgattchildren
            // Step 7.
            BluetoothResponse::GetDescriptors(descriptors_vec, single) => {
                if single {
                    promise.resolve_native(
                        &device.get_or_create_descriptor(&descriptors_vec[0], self),
                    );
                    return;
                }
                let mut descriptors = vec![];
                for descriptor in descriptors_vec {
                    let bt_descriptor = device.get_or_create_descriptor(&descriptor, self);
                    descriptors.push(bt_descriptor);
                }
                promise.resolve_native(&descriptors);
            },
            // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-readvalue
            BluetoothResponse::ReadValue(result) => {
                // TODO: Step 5.5.1: Implement activeAlgorithms internal slot for BluetoothRemoteGATTServer.

                // Step 5.5.2.
                // TODO(#5014): Replace ByteString with ArrayBuffer when it is implemented.
                let value = ByteString::new(result);
                *self.value.borrow_mut() = Some(value.clone());

                // Step 5.5.3.
                self.upcast::<EventTarget>()
                    .fire_bubbling_event(atom!("characteristicvaluechanged"), can_gc);

                // Step 5.5.4.
                promise.resolve_native(&value);
            },
            // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-writevalue
            BluetoothResponse::WriteValue(result) => {
                // TODO: Step 7.5.1: Implement activeAlgorithms internal slot for BluetoothRemoteGATTServer.

                // Step 7.5.2.
                // TODO(#5014): Replace ByteString with an ArrayBuffer wrapped in a DataView.
                *self.value.borrow_mut() = Some(ByteString::new(result));

                // Step 7.5.3.
                promise.resolve_native(&());
            },
            // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-startnotifications
            // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-stopnotifications
            BluetoothResponse::EnableNotification(_result) => {
                // (StartNotification) TODO: Step 10:  Implement `active notification context set`
                // for BluetoothRemoteGATTCharacteristic.

                // (StartNotification) Step 11.
                // (StopNotification)  Step 5.
                promise.resolve_native(self);
            },
            _ => promise.reject_error(Error::Type("Something went wrong...".to_owned())),
        }
    }
}
