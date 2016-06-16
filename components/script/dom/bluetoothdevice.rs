/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::BluetoothDeviceBinding;
use dom::bindings::codegen::Bindings::BluetoothDeviceBinding::BluetoothDeviceMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root, MutHeap, MutNullableHeap};
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::bluetoothadvertisingdata::BluetoothAdvertisingData;
use dom::bluetoothremotegattserver::BluetoothRemoteGATTServer;

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothdevice
#[dom_struct]
pub struct BluetoothDevice {
    reflector_: Reflector,
    id: DOMString,
    name: Option<DOMString>,
    adData: MutHeap<JS<BluetoothAdvertisingData>>,
    gatt: MutNullableHeap<JS<BluetoothRemoteGATTServer>>,
}

impl BluetoothDevice {
    pub fn new_inherited(id: DOMString,
                         name: Option<DOMString>,
                         adData: &BluetoothAdvertisingData)
                         -> BluetoothDevice {
        BluetoothDevice {
            reflector_: Reflector::new(),
            id: id,
            name: name,
            adData: MutHeap::new(adData),
            gatt: Default::default(),
        }
    }

    pub fn new(global: GlobalRef,
               id: DOMString,
               name: Option<DOMString>,
               adData: &BluetoothAdvertisingData)
               -> Root<BluetoothDevice> {
        reflect_dom_object(box BluetoothDevice::new_inherited(id,
                                                              name,
                                                              adData),
                           global,
                           BluetoothDeviceBinding::Wrap)
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
        self.adData.get()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothdevice-gatt
    fn Gatt(&self) -> Root<BluetoothRemoteGATTServer> {
        self.gatt.or_init(|| BluetoothRemoteGATTServer::new(self.global().r(), self))
    }
}
