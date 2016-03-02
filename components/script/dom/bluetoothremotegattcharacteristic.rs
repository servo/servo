/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::BluetoothRemoteGATTCharacteristicBinding;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTCharacteristicBinding::
    BluetoothRemoteGATTCharacteristicMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, MutHeap, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::ByteString;
use dom::bluetoothcharacteristicproperties::BluetoothCharacteristicProperties;
use dom::bluetoothremotegattdescriptor::BluetoothRemoteGATTDescriptor;
use dom::bluetoothremotegattservice::BluetoothRemoteGATTService;
use util::str::DOMString;

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattcharacteristic
#[dom_struct]
pub struct BluetoothRemoteGATTCharacteristic {
    reflector_: Reflector,
    service: MutHeap<JS<BluetoothRemoteGATTService>>,
    uuid: DOMString,
    properties: MutHeap<JS<BluetoothCharacteristicProperties>>,
    value: Option<ByteString>,
}

impl BluetoothRemoteGATTCharacteristic {
    pub fn new_inherited(service: &BluetoothRemoteGATTService,
                         uuid: DOMString,
                         properties: &BluetoothCharacteristicProperties)
                         -> BluetoothRemoteGATTCharacteristic {
        BluetoothRemoteGATTCharacteristic {
            reflector_: Reflector::new(),
            service: MutHeap::new(service),
            uuid: uuid,
            properties: MutHeap::new(properties),
            value: None,
        }
    }

    pub fn new(global: GlobalRef,
               service: &BluetoothRemoteGATTService,
               uuid: DOMString,
               properties: &BluetoothCharacteristicProperties)
               -> Root<BluetoothRemoteGATTCharacteristic> {
        reflect_dom_object(box BluetoothRemoteGATTCharacteristic::new_inherited(service,
                                                                                uuid,
                                                                                properties),
                            global,
                            BluetoothRemoteGATTCharacteristicBinding::Wrap)
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

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-getdescriptor
    fn GetDescriptor(&self) -> Option<Root<BluetoothRemoteGATTDescriptor>> {
        //UNIMPLEMENTED
        None
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-value
    fn GetValue(&self) -> Option<ByteString> {
        self.value.clone()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattcharacteristic-readvalue
    fn ReadValue(&self) -> ByteString {
        //UNIMPLEMENTED
        ByteString::new(vec!())
    }
}
