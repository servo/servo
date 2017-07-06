/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::nonzero::NonZero;
use dom::bindings::codegen::Bindings::GamepadBinding;
use dom::bindings::codegen::Bindings::GamepadBinding::GamepadMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root};
use dom::bindings::num::Finite;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::event::Event;
use dom::eventtarget::EventTarget;
use dom::gamepadbuttonlist::GamepadButtonList;
use dom::gamepadevent::{GamepadEvent, GamepadEventType};
use dom::globalscope::GlobalScope;
use dom::vrpose::VRPose;
use dom_struct::dom_struct;
use js::jsapi::{Heap, JSContext, JSObject};
use js::typedarray::{Float64Array, CreateWith};
use std::cell::Cell;
use std::ptr;
use webvr_traits::{WebVRGamepadData, WebVRGamepadHand, WebVRGamepadState};

#[dom_struct]
pub struct Gamepad {
    reflector_: Reflector,
    gamepad_id: u32,
    id: String,
    index: Cell<i32>,
    connected: Cell<bool>,
    timestamp: Cell<f64>,
    mapping_type: String,
    axes: Heap<*mut JSObject>,
    buttons: JS<GamepadButtonList>,
    pose: Option<JS<VRPose>>,
    #[ignore_heap_size_of = "Defined in rust-webvr"]
    hand: WebVRGamepadHand,
    display_id: u32
}

impl Gamepad {
    fn new_inherited(gamepad_id: u32,
                     id: String,
                     index: i32,
                     connected: bool,
                     timestamp: f64,
                     mapping_type: String,
                     buttons: &GamepadButtonList,
                     pose: Option<&VRPose>,
                     hand: WebVRGamepadHand,
                     display_id: u32) -> Gamepad {
        Self {
            reflector_: Reflector::new(),
            gamepad_id: gamepad_id,
            id: id,
            index: Cell::new(index),
            connected: Cell::new(connected),
            timestamp: Cell::new(timestamp),
            mapping_type: mapping_type,
            axes: Heap::default(),
            buttons: JS::from_ref(buttons),
            pose: pose.map(JS::from_ref),
            hand: hand,
            display_id: display_id
        }
    }

    #[allow(unsafe_code)]
    pub fn new_from_vr(global: &GlobalScope,
                       index: i32,
                       data: &WebVRGamepadData,
                       state: &WebVRGamepadState) -> Root<Gamepad> {
        let buttons = GamepadButtonList::new_from_vr(&global, &state.buttons);
        let pose = VRPose::new(&global, &state.pose);

        let gamepad = reflect_dom_object(box Gamepad::new_inherited(state.gamepad_id,
                                                                    data.name.clone(),
                                                                    index,
                                                                    state.connected,
                                                                    state.timestamp,
                                                                    "".into(),
                                                                    &buttons,
                                                                    Some(&pose),
                                                                    data.hand.clone(),
                                                                    data.display_id),
                                         global,
                                         GamepadBinding::Wrap);

        let cx = global.get_cx();
        rooted!(in (cx) let mut array = ptr::null_mut());
        unsafe {
            let _ = Float64Array::create(cx, CreateWith::Slice(&state.axes), array.handle_mut());
        }
        gamepad.axes.set(array.get());

        gamepad
    }
}

impl GamepadMethods for Gamepad {
    // https://w3c.github.io/gamepad/#dom-gamepad-id
    fn Id(&self) -> DOMString {
        DOMString::from(self.id.clone())
    }

    // https://w3c.github.io/gamepad/#dom-gamepad-index
    fn Index(&self) -> i32 {
        self.index.get()
    }

    // https://w3c.github.io/gamepad/#dom-gamepad-connected
    fn Connected(&self) -> bool {
        self.connected.get()
    }

    // https://w3c.github.io/gamepad/#dom-gamepad-timestamp
    fn Timestamp(&self) -> Finite<f64> {
        Finite::wrap(self.timestamp.get())
    }

    // https://w3c.github.io/gamepad/#dom-gamepad-mapping
    fn Mapping(&self) -> DOMString {
        DOMString::from(self.mapping_type.clone())
    }

    #[allow(unsafe_code)]
    // https://w3c.github.io/gamepad/#dom-gamepad-axes
    unsafe fn Axes(&self, _cx: *mut JSContext) -> NonZero<*mut JSObject> {
        NonZero::new(self.axes.get())
    }

    // https://w3c.github.io/gamepad/#dom-gamepad-buttons
    fn Buttons(&self) -> Root<GamepadButtonList> {
        Root::from_ref(&*self.buttons)
    }

    // https://w3c.github.io/gamepad/extensions.html#gamepadhand-enum
    fn Hand(&self) -> DOMString {
        let value = match self.hand {
            WebVRGamepadHand::Unknown => "",
            WebVRGamepadHand::Left => "left",
            WebVRGamepadHand::Right => "right"
        };
        value.into()
    }

    // https://w3c.github.io/gamepad/extensions.html#dom-gamepad-pose
    fn GetPose(&self) -> Option<Root<VRPose>> {
        self.pose.as_ref().map(|p| Root::from_ref(&**p))
    }

    // https://w3c.github.io/webvr/spec/1.1/#gamepad-getvrdisplays-attribute
    fn DisplayId(&self) -> u32 {
        self.display_id
    }
}

impl Gamepad {
    #[allow(unsafe_code)]
    pub fn update_from_vr(&self, state: &WebVRGamepadState) {
        self.timestamp.set(state.timestamp);
        unsafe {
            let cx = self.global().get_cx();
            typedarray!(in(cx) let axes: Float64Array = self.axes.get());
            if let Ok(mut array) = axes {
                array.update(&state.axes);
            }
        }
        self.buttons.sync_from_vr(&state.buttons);
        if let Some(ref pose) = self.pose {
            pose.update(&state.pose);
        }
        self.update_connected(state.connected);
    }

    pub fn gamepad_id(&self) -> u32 {
        self.gamepad_id
    }

    pub fn update_connected(&self, connected: bool) {
        if self.connected.get() == connected {
            return;
        }
        self.connected.set(connected);

        let event_type = if connected {
            GamepadEventType::Connected
        } else {
            GamepadEventType::Disconnected
        };

        self.notify_event(event_type);
    }

    pub fn update_index(&self, index: i32) {
        self.index.set(index);
    }

    pub fn notify_event(&self, event_type: GamepadEventType) {
        let event = GamepadEvent::new_with_type(&self.global(), event_type, &self);
        event.upcast::<Event>().fire(self.global().as_window().upcast::<EventTarget>());
    }
}
