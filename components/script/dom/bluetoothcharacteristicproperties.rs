/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::BluetoothCharacteristicPropertiesBinding;
use dom::bindings::codegen::Bindings::BluetoothCharacteristicPropertiesBinding::
    BluetoothCharacteristicPropertiesMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};

// https://webbluetoothcg.github.io/web-bluetooth/#characteristicproperties
 #[dom_struct]
pub struct BluetoothCharacteristicProperties {
    reflector_: Reflector,
    broadcast: bool,
    read: bool,
    writeWithoutResponse: bool,
    write: bool,
    notify: bool,
    indicate: bool,
    authenticatedSignedWrites: bool,
    reliableWrite: bool,
    writableAuxiliaries: bool,
}

impl BluetoothCharacteristicProperties {
    pub fn new_inherited(broadcast: bool,
                         read: bool,
                         writeWithoutResponse: bool,
                         write: bool,
                         notify: bool,
                         indicate: bool,
                         authenticatedSignedWrites: bool,
                         reliableWrite: bool,
                         writableAuxiliaries: bool)
                         -> BluetoothCharacteristicProperties {
        BluetoothCharacteristicProperties {
            reflector_: Reflector::new(),
            broadcast: broadcast,
            read: read,
            writeWithoutResponse: writeWithoutResponse,
            write: write,
            notify: notify,
            indicate: indicate,
            authenticatedSignedWrites: authenticatedSignedWrites,
            reliableWrite: reliableWrite,
            writableAuxiliaries: writableAuxiliaries,
        }
    }

    pub fn new(global: GlobalRef,
               broadcast: bool,
               read: bool,
               writeWithoutResponse: bool,
               write: bool,
               notify: bool,
               indicate: bool,
               authenticatedSignedWrites: bool,
               reliableWrite: bool,
               writableAuxiliaries: bool)
               -> Root<BluetoothCharacteristicProperties> {
        reflect_dom_object(box BluetoothCharacteristicProperties::new_inherited(broadcast,
                                                                                read,
                                                                                writeWithoutResponse,
                                                                                write,
                                                                                notify,
                                                                                indicate,
                                                                                authenticatedSignedWrites,
                                                                                reliableWrite,
                                                                                writableAuxiliaries),
                           global,
                           BluetoothCharacteristicPropertiesBinding::Wrap)
        }
    }

impl BluetoothCharacteristicPropertiesMethods for BluetoothCharacteristicProperties {
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothcharacteristicproperties-broadcast
    fn Broadcast(&self) -> bool {
        self.broadcast
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothcharacteristicproperties-read
    fn Read(&self) -> bool {
        self.read
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothcharacteristicproperties-writewithoutresponse
    fn WriteWithoutResponse(&self) -> bool {
        self.writeWithoutResponse
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothcharacteristicproperties-write
    fn Write(&self) -> bool {
        self.write
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothcharacteristicproperties-notify
    fn Notify(&self) -> bool {
        self.notify
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothcharacteristicproperties-indicate
    fn Indicate(&self) -> bool {
        self.indicate
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothcharacteristicproperties-authenticatedsignedwrites
    fn AuthenticatedSignedWrites(&self) -> bool {
        self.authenticatedSignedWrites
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothcharacteristicproperties-reliablewrite
    fn ReliableWrite(&self) -> bool {
        self.reliableWrite
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothcharacteristicproperties-writableauxiliaries
    fn WritableAuxiliaries(&self) -> bool {
        self.writableAuxiliaries
    }
}
