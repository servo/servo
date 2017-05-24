/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::VRBinding;
use dom::bindings::codegen::Bindings::VRBinding::VRMethods;
use dom::bindings::codegen::Bindings::VRDisplayBinding::VRDisplayMethods;
use dom::bindings::error::Error;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::event::Event;
use dom::eventtarget::EventTarget;
use dom::gamepad::Gamepad;
use dom::gamepadevent::GamepadEventType;
use dom::globalscope::GlobalScope;
use dom::promise::Promise;
use dom::vrdisplay::VRDisplay;
use dom::vrdisplayevent::VRDisplayEvent;
use dom_struct::dom_struct;
use ipc_channel::ipc;
use ipc_channel::ipc::IpcSender;
use std::rc::Rc;
use webvr_traits::{WebVRDisplayData, WebVRDisplayEvent, WebVREvent, WebVRMsg};
use webvr_traits::{WebVRGamepadData, WebVRGamepadEvent, WebVRGamepadState};

#[dom_struct]
pub struct VR {
    eventtarget: EventTarget,
    displays: DOMRefCell<Vec<JS<VRDisplay>>>,
    gamepads: DOMRefCell<Vec<JS<Gamepad>>>
}

impl VR {
    fn new_inherited() -> VR {
        VR {
            eventtarget: EventTarget::new_inherited(),
            displays: DOMRefCell::new(Vec::new()),
            gamepads: DOMRefCell::new(Vec::new()),
        }
    }

    pub fn new(global: &GlobalScope) -> Root<VR> {
        let root = reflect_dom_object(box VR::new_inherited(),
                           global,
                           VRBinding::Wrap);
        root.register();
        root
    }
}

impl Drop for VR {
    fn drop(&mut self) {
        self.unregister();
    }
}

impl VRMethods for VR {
    #[allow(unrooted_must_root)]
    // https://w3c.github.io/webvr/#interface-navigator
    fn GetDisplays(&self) -> Rc<Promise> {
        let promise = Promise::new(&self.global());

        if let Some(webvr_thread) = self.webvr_thread() {
            let (sender, receiver) = ipc::channel().unwrap();
            webvr_thread.send(WebVRMsg::GetDisplays(sender)).unwrap();
            match receiver.recv().unwrap() {
                Ok(displays) => {
                    // Sync displays
                    for display in displays {
                        self.sync_display(&display);
                    }
                },
                Err(e) => {
                    promise.reject_native(promise.global().get_cx(), &e);
                    return promise;
                }
            }
        } else {
            // WebVR spec: The Promise MUST be rejected if WebVR is not enabled/supported.
            promise.reject_error(promise.global().get_cx(), Error::Security);
            return promise;
        }

        // convert from JS to Root
        let displays: Vec<Root<VRDisplay>> = self.displays.borrow().iter()
                                                          .map(|d| Root::from_ref(&**d))
                                                          .collect();
        promise.resolve_native(promise.global().get_cx(), &displays);

        promise
    }
}


impl VR {
    fn webvr_thread(&self) -> Option<IpcSender<WebVRMsg>> {
        self.global().as_window().webvr_thread()
    }

    fn find_display(&self, display_id: u32) -> Option<Root<VRDisplay>> {
        self.displays.borrow()
                     .iter()
                     .find(|d| d.DisplayId() == display_id)
                     .map(|d| Root::from_ref(&**d))
    }

    fn register(&self) {
        if let Some(webvr_thread) = self.webvr_thread() {
             let msg = WebVRMsg::RegisterContext(self.global().pipeline_id());
             webvr_thread.send(msg).unwrap();
        }
    }

    fn unregister(&self) {
        if let Some(webvr_thread) = self.webvr_thread() {
             let msg = WebVRMsg::UnregisterContext(self.global().pipeline_id());
             webvr_thread.send(msg).unwrap();
        }
    }

    fn sync_display(&self, display: &WebVRDisplayData) -> Root<VRDisplay> {
        if let Some(existing) = self.find_display(display.display_id) {
            existing.update_display(&display);
            existing
        } else {
            let root = VRDisplay::new(&self.global(), display.clone());
            self.displays.borrow_mut().push(JS::from_ref(&*root));
            root
        }
    }

    fn handle_display_event(&self, event: WebVRDisplayEvent) {
        match event {
            WebVRDisplayEvent::Connect(ref display) => {
                let display = self.sync_display(&display);
                display.handle_webvr_event(&event);
                self.notify_display_event(&display, &event);
            },
            WebVRDisplayEvent::Disconnect(id) => {
                if let Some(display) = self.find_display(id) {
                    display.handle_webvr_event(&event);
                    self.notify_display_event(&display, &event);
                }
            },
            WebVRDisplayEvent::Activate(ref display, _) |
            WebVRDisplayEvent::Deactivate(ref display, _) |
            WebVRDisplayEvent::Blur(ref display) |
            WebVRDisplayEvent::Focus(ref display) |
            WebVRDisplayEvent::PresentChange(ref display, _) |
            WebVRDisplayEvent::Change(ref display) => {
                let display = self.sync_display(&display);
                display.handle_webvr_event(&event);
            },
            WebVRDisplayEvent::Pause(id) |
            WebVRDisplayEvent::Resume(id) |
            WebVRDisplayEvent::Exit(id) => {
                if let Some(display) = self.find_display(id) {
                    display.handle_webvr_event(&event);
                }
            }
        };
    }

    fn handle_gamepad_event(&self, event: WebVRGamepadEvent) {
        match event {
            WebVRGamepadEvent::Connect(data, state) => {
                if let Some(gamepad) = self.find_gamepad(state.gamepad_id) {
                    gamepad.update_from_vr(&state);
                } else {
                    // new gamepad
                    self.sync_gamepad(Some(data), &state);
                }
            },
            WebVRGamepadEvent::Disconnect(id) => {
                if let Some(gamepad) = self.find_gamepad(id) {
                    gamepad.update_connected(false);
                }
            }
        };
    }

    pub fn handle_webvr_event(&self, event: WebVREvent) {
        match event {
            WebVREvent::Display(event) => {
                self.handle_display_event(event);
            },
            WebVREvent::Gamepad(event) => {
                self.handle_gamepad_event(event);
            }
        };
    }

    pub fn handle_webvr_events(&self, events: Vec<WebVREvent>) {
        for event in events {
            self.handle_webvr_event(event);
        }
    }

    fn notify_display_event(&self, display: &VRDisplay, event: &WebVRDisplayEvent) {
        let event = VRDisplayEvent::new_from_webvr(&self.global(), &display, &event);
        event.upcast::<Event>().fire(self.upcast());
    }
}

// Gamepad
impl VR {
    fn find_gamepad(&self, gamepad_id: u32) -> Option<Root<Gamepad>> {
        self.gamepads.borrow()
                     .iter()
                     .find(|g| g.gamepad_id() == gamepad_id)
                     .map(|g| Root::from_ref(&**g))
    }

    fn sync_gamepad(&self, data: Option<WebVRGamepadData>, state: &WebVRGamepadState) {
        if let Some(existing) = self.find_gamepad(state.gamepad_id) {
            existing.update_from_vr(&state);
        } else {
            let index = self.gamepads.borrow().len();
            let data = data.unwrap_or_default();
            let root = Gamepad::new_from_vr(&self.global(),
                                            index as i32,
                                            &data,
                                            &state);
            self.gamepads.borrow_mut().push(JS::from_ref(&*root));
            if state.connected {
                root.notify_event(GamepadEventType::Connected);
            }
        }
    }

    // Gamepads are synced immediately in response to the API call.
    // The current approach allows the to sample gamepad state multiple times per frame. This
    // guarantees that the gamepads always have a valid state and can be very useful for
    // motion capture or drawing applications.
    pub fn get_gamepads(&self) -> Vec<Root<Gamepad>> {
        if let Some(wevbr_sender) = self.webvr_thread() {
            let (sender, receiver) = ipc::channel().unwrap();
            let synced_ids = self.gamepads.borrow().iter().map(|g| g.gamepad_id()).collect();
            wevbr_sender.send(WebVRMsg::GetGamepads(synced_ids, sender)).unwrap();
            match receiver.recv().unwrap() {
                Ok(gamepads) => {
                    // Sync displays
                    for gamepad in gamepads {
                        self.sync_gamepad(gamepad.0, &gamepad.1);
                    }
                },
                Err(_) => {}
            }
        }

        // We can add other not VR related gamepad providers here
        self.gamepads.borrow().iter()
                              .map(|g| Root::from_ref(&**g))
                              .collect()
    }
}
