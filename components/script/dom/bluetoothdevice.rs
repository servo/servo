/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::BluetoothDeviceBinding;
use dom::bindings::codegen::Bindings::BluetoothDeviceBinding::BluetoothDeviceMethods;
use dom::bindings::js::{JS, Root, MutHeap, MutNullableHeap};
use dom::bindings::reflector::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::bluetooth::Bluetooth;
use dom::bluetoothadvertisingdata::BluetoothAdvertisingData;
use dom::bluetoothremotegattserver::BluetoothRemoteGATTServer;
use dom::globalscope::GlobalScope;

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothdevice
#[dom_struct]
pub struct BluetoothDevice {
    reflector_: Reflector,
    id: DOMString,
    name: Option<DOMString>,
    ad_data: MutHeap<JS<BluetoothAdvertisingData>>,
    gatt: MutNullableHeap<JS<BluetoothRemoteGATTServer>>,
    context: MutHeap<JS<Bluetooth>>,
}

impl BluetoothDevice {
    pub fn new_inherited(id: DOMString,
                         name: Option<DOMString>,
                         ad_data: &BluetoothAdvertisingData,
                         context: &Bluetooth)
                         -> BluetoothDevice {
        BluetoothDevice {
            reflector_: Reflector::new(),
            id: id,
            name: name,
            ad_data: MutHeap::new(ad_data),
            gatt: Default::default(),
            context: MutHeap::new(context),
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

    pub fn get_context(&self) -> Root<Bluetooth> {
        self.context.get()
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
        self.gatt.or_init(|| {
            BluetoothRemoteGATTServer::new(&self.global(), self)
        })
    }
}
