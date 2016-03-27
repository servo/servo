/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::BluetoothAdvertisingDataBinding;
use dom::bindings::codegen::Bindings::BluetoothAdvertisingDataBinding::BluetoothAdvertisingDataMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothadvertisingdata
#[dom_struct]
pub struct BluetoothAdvertisingData {
    reflector_: Reflector,
    appearance: u16,
    txPower: i8,
    rssi: i8,
}

impl BluetoothAdvertisingData {
    pub fn new_inherited(appearance: u16, txPower: i8, rssi: i8) -> BluetoothAdvertisingData {
        BluetoothAdvertisingData {
            reflector_: Reflector::new(),
            appearance: appearance,
            txPower: txPower,
            rssi: rssi,
        }
    }

    pub fn new(global: GlobalRef, appearance: u16, txPower: i8, rssi: i8) -> Root<BluetoothAdvertisingData> {
        reflect_dom_object(box BluetoothAdvertisingData::new_inherited(appearance,
                                                                       txPower,
                                                                       rssi),
                           global,
                           BluetoothAdvertisingDataBinding::Wrap)
    }
}

impl BluetoothAdvertisingDataMethods for BluetoothAdvertisingData {
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothadvertisingdata-appearance
    fn GetAppearance(&self) -> Option<u16> {
        Some(self.appearance)
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothadvertisingdata-txpower
    fn GetTxPower(&self) -> Option<i8> {
        Some(self.txPower)
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothadvertisingdata-rssi
    fn GetRssi(&self) -> Option<i8> {
        Some(self.rssi)
    }
}
