/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::BluetoothCharacteristicPropertiesBinding;
use dom::bindings::codegen::Bindings::BluetoothCharacteristicPropertiesBinding::
    BluetoothCharacteristicPropertiesMethods;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;

// https://webbluetoothcg.github.io/web-bluetooth/#characteristicproperties
 #[dom_struct]
pub struct BluetoothCharacteristicProperties {
    reflector_: Reflector,
    broadcast: bool,
    read: bool,
    write_without_response: bool,
    write: bool,
    notify: bool,
    indicate: bool,
    authenticated_signed_writes: bool,
    reliable_write: bool,
    writable_auxiliaries: bool,
}

impl BluetoothCharacteristicProperties {
    pub fn new_inherited(broadcast: bool,
                         read: bool,
                         write_without_response: bool,
                         write: bool,
                         notify: bool,
                         indicate: bool,
                         authenticated_signed_writes: bool,
                         reliable_write: bool,
                         writable_auxiliaries: bool)
                         -> BluetoothCharacteristicProperties {
        BluetoothCharacteristicProperties {
            reflector_: Reflector::new(),
            broadcast: broadcast,
            read: read,
            write_without_response: write_without_response,
            write: write,
            notify: notify,
            indicate: indicate,
            authenticated_signed_writes: authenticated_signed_writes,
            reliable_write: reliable_write,
            writable_auxiliaries: writable_auxiliaries,
        }
    }

    pub fn new(global: &GlobalScope,
               broadcast: bool,
               read: bool,
               writeWithoutResponse: bool,
               write: bool,
               notify: bool,
               indicate: bool,
               authenticatedSignedWrites: bool,
               reliableWrite: bool,
               writableAuxiliaries: bool)
               -> DomRoot<BluetoothCharacteristicProperties> {
        reflect_dom_object(
            Box::new(BluetoothCharacteristicProperties::new_inherited(
                broadcast,
                read,
                writeWithoutResponse,
                write,
                notify,
                indicate,
                authenticatedSignedWrites,
                reliableWrite,
                writableAuxiliaries
            )),
            global,
            BluetoothCharacteristicPropertiesBinding::Wrap
        )
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
        self.write_without_response
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
        self.authenticated_signed_writes
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothcharacteristicproperties-reliablewrite
    fn ReliableWrite(&self) -> bool {
        self.reliable_write
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothcharacteristicproperties-writableauxiliaries
    fn WritableAuxiliaries(&self) -> bool {
        self.writable_auxiliaries
    }
}
