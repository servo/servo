/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::BluetoothAdvertisingEventBinding::{self, BluetoothAdvertisingEventInit};
use dom::bindings::codegen::Bindings::BluetoothAdvertisingEventBinding::BluetoothAdvertisingEventMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventBinding::EventMethods;
use dom::bindings::error::Fallible;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root, RootedReference};
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::DOMString;
use dom::bluetoothdevice::BluetoothDevice;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::globalscope::GlobalScope;
use dom::window::Window;
use dom_struct::dom_struct;
use servo_atoms::Atom;

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothadvertisingevent
#[dom_struct]
pub struct BluetoothAdvertisingEvent {
    event: Event,
    device: JS<BluetoothDevice>,
    name: Option<DOMString>,
    appearance: Option<u16>,
    tx_power: Option<i8>,
    rssi: Option<i8>,
}

impl BluetoothAdvertisingEvent {
    pub fn new_inherited(device: &BluetoothDevice,
                         name: Option<DOMString>,
                         appearance: Option<u16>,
                         tx_power: Option<i8>,
                         rssi: Option<i8>)
                         -> BluetoothAdvertisingEvent {
        BluetoothAdvertisingEvent {
            event: Event::new_inherited(),
            device: JS::from_ref(device),
            name: name,
            appearance: appearance,
            tx_power: tx_power,
            rssi: rssi,
        }
    }

    pub fn new(global: &GlobalScope,
               type_: Atom,
               bubbles: EventBubbles,
               cancelable: EventCancelable,
               device: &BluetoothDevice,
               name: Option<DOMString>,
               appearance: Option<u16>,
               txPower: Option<i8>,
               rssi: Option<i8>)
               -> Root<BluetoothAdvertisingEvent> {
        let ev = reflect_dom_object(box BluetoothAdvertisingEvent::new_inherited(device,
                                                                                 name,
                                                                                 appearance,
                                                                                 txPower,
                                                                                 rssi),
                                    global,
                                    BluetoothAdvertisingEventBinding::Wrap);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bool::from(bubbles), bool::from(cancelable));
        }
        ev
    }

    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothadvertisingevent-bluetoothadvertisingevent
    pub fn Constructor(window: &Window,
                       type_: DOMString,
                       init: &BluetoothAdvertisingEventInit)
                       -> Fallible<Root<BluetoothAdvertisingEvent>> {
        let global = window.upcast::<GlobalScope>();
        let device = init.device.r();
        let name = init.name.clone();
        let appearance = init.appearance.clone();
        let txPower = init.txPower.clone();
        let rssi = init.rssi.clone();
        let bubbles = EventBubbles::from(init.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.cancelable);
        Ok(BluetoothAdvertisingEvent::new(global,
                                          Atom::from(type_),
                                          bubbles,
                                          cancelable,
                                          device,
                                          name,
                                          appearance,
                                          txPower,
                                          rssi))
    }
}

impl BluetoothAdvertisingEventMethods for BluetoothAdvertisingEvent {
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothadvertisingevent-device
    fn Device(&self) -> Root<BluetoothDevice> {
        Root::from_ref(&*self.device)
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
