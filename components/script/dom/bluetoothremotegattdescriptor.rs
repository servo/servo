/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::BluetoothRemoteGATTDescriptorBinding;
use dom::bindings::codegen::Bindings::BluetoothRemoteGATTDescriptorBinding::BluetoothRemoteGATTDescriptorMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, MutHeap, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::ByteString;
use dom::bluetoothremotegattcharacteristic::BluetoothRemoteGATTCharacteristic;
use util::str::DOMString;

// http://webbluetoothcg.github.io/web-bluetooth/#bluetoothremotegattdescriptor
#[dom_struct]
pub struct BluetoothRemoteGATTDescriptor {
    reflector_: Reflector,
    characteristic: MutHeap<JS<BluetoothRemoteGATTCharacteristic>>,
    uuid: DOMString,
    value: Option<ByteString>,
}

impl BluetoothRemoteGATTDescriptor {
    pub fn new_inherited(characteristic: &BluetoothRemoteGATTCharacteristic,
                         uuid: DOMString)
                         -> BluetoothRemoteGATTDescriptor {
        BluetoothRemoteGATTDescriptor {
            reflector_: Reflector::new(),
            characteristic: MutHeap::new(characteristic),
            uuid: uuid,
            value: None,
        }
    }

    pub fn new(global: GlobalRef,
               characteristic: &BluetoothRemoteGATTCharacteristic,
               uuid: DOMString)
               -> Root<BluetoothRemoteGATTDescriptor>{
        reflect_dom_object(box BluetoothRemoteGATTDescriptor::new_inherited(characteristic,
                                                                            uuid),
                            global,
                            BluetoothRemoteGATTDescriptorBinding::Wrap)
    }
}

impl BluetoothRemoteGATTDescriptorMethods for BluetoothRemoteGATTDescriptor {

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattdescriptor-characteristic
    fn Characteristic(&self) -> Root<BluetoothRemoteGATTCharacteristic> {
       self.characteristic.get()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattdescriptor-uuid
    fn Uuid(&self) -> DOMString {
        self.uuid.clone()
    }

     // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattdescriptor-value
    fn GetValue(&self) -> Option<ByteString> {
        self.value.clone()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothremotegattdescriptor-readvalue
    fn ReadValue(&self) -> ByteString {
        //UNIMPLEMENTED
        ByteString::new(vec!())
    }
}
