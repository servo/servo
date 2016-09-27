/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::BluetoothAdvertisingDataBinding;
use dom::bindings::codegen::Bindings::BluetoothAdvertisingDataBinding::BluetoothAdvertisingDataMethods;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::globalscope::GlobalScope;

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothadvertisingdata
#[dom_struct]
pub struct BluetoothAdvertisingData {
    reflector_: Reflector,
    appearance: Option<u16>,
    tx_power: Option<i8>,
    rssi: Option<i8>,
}

impl BluetoothAdvertisingData {
    pub fn new_inherited(appearance: Option<u16>,
                         tx_power: Option<i8>,
                         rssi: Option<i8>)
                         -> BluetoothAdvertisingData {
        BluetoothAdvertisingData {
            reflector_: Reflector::new(),
            appearance: appearance,
            tx_power: tx_power,
            rssi: rssi,
        }
    }

    pub fn new(global: &GlobalScope,
               appearance: Option<u16>,
               txPower: Option<i8>,
               rssi: Option<i8>)
               -> Root<BluetoothAdvertisingData> {
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
        self.appearance
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothadvertisingdata-txpower
    fn GetTxPower(&self) -> Option<i8> {
        self.tx_power
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothadvertisingdata-rssi
    fn GetRssi(&self) -> Option<i8> {
        self.rssi
    }
}
