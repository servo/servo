/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bluetooth_traits::{BluetoothCharacteristicMsg, BluetoothDescriptorMsg, BluetoothServiceMsg};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BluetoothDeviceBinding;
use dom::bindings::codegen::Bindings::BluetoothDeviceBinding::BluetoothDeviceMethods;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServerBinding::BluetoothRemoteGATTServerMethods;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::js::{MutJS, MutNullableJS, Root};
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::bluetooth::Bluetooth;
use dom::bluetoothadvertisingdata::BluetoothAdvertisingData;
use dom::bluetoothcharacteristicproperties::BluetoothCharacteristicProperties;
use dom::bluetoothremotegattcharacteristic::BluetoothRemoteGATTCharacteristic;
use dom::bluetoothremotegattdescriptor::BluetoothRemoteGATTDescriptor;
use dom::bluetoothremotegattserver::BluetoothRemoteGATTServer;
use dom::bluetoothremotegattservice::BluetoothRemoteGATTService;
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;
use std::collections::HashMap;


// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothdevice
#[dom_struct]
pub struct BluetoothDevice {
    eventtarget: EventTarget,
    id: DOMString,
    name: Option<DOMString>,
    ad_data: MutJS<BluetoothAdvertisingData>,
    gatt: MutNullableJS<BluetoothRemoteGATTServer>,
    context: MutJS<Bluetooth>,
    attribute_instance_map: (DOMRefCell<HashMap<String, MutJS<BluetoothRemoteGATTService>>>,
                             DOMRefCell<HashMap<String, MutJS<BluetoothRemoteGATTCharacteristic>>>,
                             DOMRefCell<HashMap<String, MutJS<BluetoothRemoteGATTDescriptor>>>),
}

impl BluetoothDevice {
    pub fn new_inherited(id: DOMString,
                         name: Option<DOMString>,
                         ad_data: &BluetoothAdvertisingData,
                         context: &Bluetooth)
                         -> BluetoothDevice {
        BluetoothDevice {
            eventtarget: EventTarget::new_inherited(),
            id: id,
            name: name,
            ad_data: MutJS::new(ad_data),
            gatt: Default::default(),
            context: MutJS::new(context),
            attribute_instance_map: (DOMRefCell::new(HashMap::new()),
                                     DOMRefCell::new(HashMap::new()),
                                     DOMRefCell::new(HashMap::new())),
        }
    }

    pub fn new(global: &GlobalScope,
               id: DOMString,
               name: Option<DOMString>,
               adData: &BluetoothAdvertisingData,
               context: &Bluetooth)
               -> Root<BluetoothDevice> {
        reflect_dom_object(box BluetoothDevice::new_inherited(id,
                                                              name,
                                                              adData,
                                                              context),
                           global,
                           BluetoothDeviceBinding::Wrap)
    }

    pub fn get_or_create_service(&self,
                                 service: &BluetoothServiceMsg,
                                 server: &BluetoothRemoteGATTServer)
                                 -> Root<BluetoothRemoteGATTService> {
        let (ref service_map_ref, _, _) = self.attribute_instance_map;
        let mut service_map = service_map_ref.borrow_mut();
        if let Some(existing_service) = service_map.get(&service.instance_id) {
            return existing_service.get();
        }
        let bt_service = BluetoothRemoteGATTService::new(&server.global(),
                                                         &server.Device(),
                                                         DOMString::from(service.uuid.clone()),
                                                         service.is_primary,
                                                         service.instance_id.clone());
        service_map.insert(service.instance_id.clone(), MutJS::new(&bt_service));
        return bt_service;
    }

    pub fn get_or_create_characteristic(&self,
                                        characteristic: &BluetoothCharacteristicMsg,
                                        service: &BluetoothRemoteGATTService)
                                        -> Root<BluetoothRemoteGATTCharacteristic> {
        let (_, ref characteristic_map_ref, _) = self.attribute_instance_map;
        let mut characteristic_map = characteristic_map_ref.borrow_mut();
        if let Some(existing_characteristic) = characteristic_map.get(&characteristic.instance_id) {
            return existing_characteristic.get();
        }
        let properties =
            BluetoothCharacteristicProperties::new(&service.global(),
                                                   characteristic.broadcast,
                                                   characteristic.read,
                                                   characteristic.write_without_response,
                                                   characteristic.write,
                                                   characteristic.notify,
                                                   characteristic.indicate,
                                                   characteristic.authenticated_signed_writes,
                                                   characteristic.reliable_write,
                                                   characteristic.writable_auxiliaries);
        let bt_characteristic = BluetoothRemoteGATTCharacteristic::new(&service.global(),
                                                                       service,
                                                                       DOMString::from(characteristic.uuid.clone()),
                                                                       &properties,
                                                                       characteristic.instance_id.clone());
        characteristic_map.insert(characteristic.instance_id.clone(), MutJS::new(&bt_characteristic));
        return bt_characteristic;
    }

    pub fn get_or_create_descriptor(&self,
                                    descriptor: &BluetoothDescriptorMsg,
                                    characteristic: &BluetoothRemoteGATTCharacteristic)
                                    -> Root<BluetoothRemoteGATTDescriptor> {
        let (_, _, ref descriptor_map_ref) = self.attribute_instance_map;
        let mut descriptor_map = descriptor_map_ref.borrow_mut();
        if let Some(existing_descriptor) = descriptor_map.get(&descriptor.instance_id) {
            return existing_descriptor.get();
        }
        let bt_descriptor = BluetoothRemoteGATTDescriptor::new(&characteristic.global(),
                                                               characteristic,
                                                               DOMString::from(descriptor.uuid.clone()),
                                                               descriptor.instance_id.clone());
        descriptor_map.insert(descriptor.instance_id.clone(), MutJS::new(&bt_descriptor));
        return bt_descriptor;
    }
}

impl BluetoothDeviceMethods for BluetoothDevice {
     // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothdevice-id
    fn Id(&self) -> DOMString {
        self.id.clone()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothdevice-name
    fn GetName(&self) -> Option<DOMString> {
        self.name.clone()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothdevice-addata
    fn AdData(&self) -> Root<BluetoothAdvertisingData> {
        self.ad_data.get()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothdevice-gatt
    fn Gatt(&self) -> Root<BluetoothRemoteGATTServer> {
        // TODO: Step 1 - 2: Implement the Permission API.
        self.gatt.or_init(|| {
            BluetoothRemoteGATTServer::new(&self.global(), self)
        })
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothdeviceeventhandlers-ongattserverdisconnected
    event_handler!(gattserverdisconnected, GetOngattserverdisconnected, SetOngattserverdisconnected);
}
