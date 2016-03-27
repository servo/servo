/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::BluetoothBinding;
use dom::bindings::codegen::Bindings::BluetoothBinding::BluetoothMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bluetoothdevice::BluetoothDevice;

// https://webbluetoothcg.github.io/web-bluetooth/#bluetooth
#[dom_struct]
pub struct Bluetooth {
    reflector_: Reflector,
}

impl Bluetooth {
    pub fn new_inherited() -> Bluetooth {
        Bluetooth {
            reflector_: Reflector::new(),
        }
    }

    pub fn new(global: GlobalRef) -> Root<Bluetooth> {
        reflect_dom_object(box Bluetooth::new_inherited(),
                           global,
                           BluetoothBinding::Wrap)
    }
}

impl BluetoothMethods for Bluetooth {

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetooth-requestdevice
    fn RequestDevice(&self) -> Option<Root<BluetoothDevice>> {
        //UNIMPLEMENTED
        None
    }
}
