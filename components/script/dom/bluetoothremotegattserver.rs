/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServerBinding;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTServerBinding::BluetoothRemoteGATTServerMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bluetoothdevice::BluetoothDevice;
use dom::bluetoothremotegattservice::BluetoothRemoteGATTService;
use std::cell::Cell;

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattserver
#[dom_struct]
pub struct BluetoothRemoteGATTServer {
    reflector_: Reflector,
    device: DOMRefCell<JS<BluetoothDevice>>,
    connected: Cell<bool>,
}

impl BluetoothRemoteGATTServer {
    pub fn new_inherited(device: &BluetoothDevice, is_connected: bool) -> BluetoothRemoteGATTServer {
        BluetoothRemoteGATTServer {
            reflector_: Reflector::new(),
            device: DOMRefCell::new(JS::from_ref(&device)),
            connected: Cell::new(is_connected),
        }
    }

    pub fn new(global: GlobalRef, device: &BluetoothDevice, connected: bool) -> Root<BluetoothRemoteGATTServer> {
        reflect_dom_object(box BluetoothRemoteGATTServer::new_inherited(
                           device,
                           connected),
        global,
        BluetoothRemoteGATTServerBinding::Wrap)
    }
}

impl BluetoothRemoteGATTServerMethods for BluetoothRemoteGATTServer {

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-device
    fn Device(&self) -> Root<BluetoothDevice> {
        Root::from_ref(&self.device.borrow())
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-connected
    fn Connected(&self) -> bool {
        self.connected.get()
    }
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-connect
    fn Connect(&self) -> Root<BluetoothRemoteGATTServer> {
        if !self.connected.get() {
            self.connected.set(true);
        }
        Root::from_ref(self)
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-disconnect
    fn Disconnect(&self) {
        self.connected.set(false);
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattserver-getprimaryservice
    fn GetPrimaryService(&self) -> Option<Root<BluetoothRemoteGATTService>> {
        //UNIMPLEMENTED
        None
    }
}
