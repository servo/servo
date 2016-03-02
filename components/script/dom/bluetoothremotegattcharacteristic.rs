/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTCharacteristicBinding;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTCharacteristicBinding::BluetoothRemoteGATTCharacteristicMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bluetoothcharacteristicproperties::BluetoothCharacteristicProperties;
use dom::bluetoothremotegattdescriptor::BluetoothRemoteGATTDescriptor;
use dom::bluetoothremotegattservice::BluetoothRemoteGATTService;
use util::str::DOMString;

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattcharacteristic
#[dom_struct]
pub struct BluetoothRemoteGATTCharacteristic {
    reflector_: Reflector,
    service: DOMRefCell<JS<BluetoothRemoteGATTService>>,
    uuid: DOMString,
    properties: DOMRefCell<JS<BluetoothCharacteristicProperties>>,
    value: u8,
}

impl BluetoothRemoteGATTCharacteristic {
    pub fn new_inherited(service: &BluetoothRemoteGATTService,
                         uuid: DOMString,
                         properties: &BluetoothCharacteristicProperties,
                         value: u8)
                         -> BluetoothRemoteGATTCharacteristic {
        BluetoothRemoteGATTCharacteristic {
            reflector_: Reflector::new(),
            service: DOMRefCell::new(JS::from_ref(&service)),
            uuid: uuid,
            properties: DOMRefCell::new(JS::from_ref(&properties)),
            value: value,
        }
    }

    pub fn new(global: GlobalRef,
               service: &BluetoothRemoteGATTService,
               uuid: DOMString,
               properties: &BluetoothCharacteristicProperties,
               value: u8)
               -> Root<BluetoothRemoteGATTCharacteristic>{
        reflect_dom_object(box BluetoothRemoteGATTCharacteristic::new_inherited(service,
                                                                                uuid,
                                                                                properties,
                                                                                value),
                            global,
                            BluetoothRemoteGATTCharacteristicBinding::Wrap)
    }
}

impl BluetoothRemoteGATTCharacteristicMethods for BluetoothRemoteGATTCharacteristic {

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-properties
    fn Properties(&self) -> Root<BluetoothCharacteristicProperties> {
        Root::from_ref(&self.properties.borrow())
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-service
    fn Service(&self) -> Root<BluetoothRemoteGATTService> {
        Root::from_ref(&self.service.borrow())
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-uuid
    fn Uuid(&self) -> DOMString {
        self.uuid.clone()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-value
    fn GetValue(&self) -> Option<u8> {
        Some(self.value)
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-getdescriptor
    fn GetDescriptor(&self) -> Option<Root<BluetoothRemoteGATTDescriptor>> {
        //UNIMPLEMENTED
        None
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-readvalue
    fn ReadValue(&self) -> u8 {
        //UNIMPLEMENTED
        0
    }
}
