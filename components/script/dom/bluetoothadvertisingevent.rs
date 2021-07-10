/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::BluetoothAdvertisingEventBinding::BluetoothAdvertisingEventInit;
use crate::dom::bindings::codegen::Bindings::BluetoothAdvertisingEventBinding::BluetoothAdvertisingEventMethods;
use crate::dom::bindings::codegen::Bindings::EventBinding::EventBinding::EventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::bluetoothdevice::BluetoothDevice;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;
use dom_struct::dom_struct;
use servo_atoms::Atom;

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothadvertisingevent
#[dom_struct]
pub struct BluetoothAdvertisingEvent {
    event: Event,
    device: Dom<BluetoothDevice>,
    name: Option<DOMString>,
    appearance: Option<u16>,
    tx_power: Option<i8>,
    rssi: Option<i8>,
}

#[allow(non_snake_case)]
impl BluetoothAdvertisingEvent {
    pub fn new_inherited(
        device: &BluetoothDevice,
        name: Option<DOMString>,
        appearance: Option<u16>,
        tx_power: Option<i8>,
        rssi: Option<i8>,
    ) -> BluetoothAdvertisingEvent {
        BluetoothAdvertisingEvent {
            event: Event::new_inherited(),
            device: Dom::from_ref(device),
            name: name,
            appearance: appearance,
            tx_power: tx_power,
            rssi: rssi,
        }
    }

    pub fn new(
        global: &GlobalScope,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        device: &BluetoothDevice,
        name: Option<DOMString>,
        appearance: Option<u16>,
        txPower: Option<i8>,
        rssi: Option<i8>,
    ) -> DomRoot<BluetoothAdvertisingEvent> {
        let ev = reflect_dom_object(
            Box::new(BluetoothAdvertisingEvent::new_inherited(
                device, name, appearance, txPower, rssi,
            )),
            global,
        );
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bool::from(bubbles), bool::from(cancelable));
        }
        ev
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothadvertisingevent-bluetoothadvertisingevent
    pub fn Constructor(
        window: &Window,
        type_: DOMString,
        init: &BluetoothAdvertisingEventInit,
    ) -> Fallible<DomRoot<BluetoothAdvertisingEvent>> {
        let global = window.upcast::<GlobalScope>();
        let name = init.name.clone();
        let appearance = init.appearance.clone();
        let txPower = init.txPower.clone();
        let rssi = init.rssi.clone();
        let bubbles = EventBubbles::from(init.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.cancelable);
        Ok(BluetoothAdvertisingEvent::new(
            global,
            Atom::from(type_),
            bubbles,
            cancelable,
            &init.device,
            name,
            appearance,
            txPower,
            rssi,
        ))
    }
}

impl BluetoothAdvertisingEventMethods for BluetoothAdvertisingEvent {
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothadvertisingevent-device
    fn Device(&self) -> DomRoot<BluetoothDevice> {
        DomRoot::from_ref(&*self.device)
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothadvertisingevent-name
    fn GetName(&self) -> Option<DOMString> {
        self.name.clone()
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothadvertisingevent-appearance
    fn GetAppearance(&self) -> Option<u16> {
        self.appearance
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothadvertisingevent-txpower
    fn GetTxPower(&self) -> Option<i8> {
        self.tx_power
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothadvertisingevent-rssi
    fn GetRssi(&self) -> Option<i8> {
        self.rssi
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
