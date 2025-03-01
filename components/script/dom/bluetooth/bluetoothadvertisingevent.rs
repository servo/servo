/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::BluetoothAdvertisingEventBinding::{
    BluetoothAdvertisingEventInit, BluetoothAdvertisingEventMethods,
};
use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::bluetoothdevice::BluetoothDevice;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

// https://webbluetoothcg.github.io/web-bluetooth/#bluetoothadvertisingevent
#[dom_struct]
pub(crate) struct BluetoothAdvertisingEvent {
    event: Event,
    device: Dom<BluetoothDevice>,
    name: Option<DOMString>,
    appearance: Option<u16>,
    tx_power: Option<i8>,
    rssi: Option<i8>,
}

#[allow(non_snake_case)]
impl BluetoothAdvertisingEvent {
    pub(crate) fn new_inherited(
        device: &BluetoothDevice,
        name: Option<DOMString>,
        appearance: Option<u16>,
        tx_power: Option<i8>,
        rssi: Option<i8>,
    ) -> BluetoothAdvertisingEvent {
        BluetoothAdvertisingEvent {
            event: Event::new_inherited(),
            device: Dom::from_ref(device),
            name,
            appearance,
            tx_power,
            rssi,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn new(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        device: &BluetoothDevice,
        name: Option<DOMString>,
        appearance: Option<u16>,
        txPower: Option<i8>,
        rssi: Option<i8>,
        can_gc: CanGc,
    ) -> DomRoot<BluetoothAdvertisingEvent> {
        let ev = reflect_dom_object_with_proto(
            Box::new(BluetoothAdvertisingEvent::new_inherited(
                device, name, appearance, txPower, rssi,
            )),
            global,
            proto,
            can_gc,
        );
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bool::from(bubbles), bool::from(cancelable));
        }
        ev
    }
}

impl BluetoothAdvertisingEventMethods<crate::DomTypeHolder> for BluetoothAdvertisingEvent {
    // https://webbluetoothcg.github.io/web-bluetooth/#dom-bluetoothadvertisingevent-bluetoothadvertisingevent
    #[allow(non_snake_case)]
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        type_: DOMString,
        init: &BluetoothAdvertisingEventInit,
    ) -> Fallible<DomRoot<BluetoothAdvertisingEvent>> {
        let name = init.name.clone();
        let appearance = init.appearance;
        let txPower = init.txPower;
        let rssi = init.rssi;
        let bubbles = EventBubbles::from(init.parent.bubbles);
        let cancelable = EventCancelable::from(init.parent.cancelable);
        Ok(BluetoothAdvertisingEvent::new(
            window.as_global_scope(),
            proto,
            Atom::from(type_),
            bubbles,
            cancelable,
            &init.device,
            name,
            appearance,
            txPower,
            rssi,
            can_gc,
        ))
    }

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
