/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::VRDisplayBinding::VRDisplayMethods;
use crate::dom::bindings::codegen::Bindings::XRBinding;
use crate::dom::bindings::codegen::Bindings::XRBinding::XRSessionCreationOptions;
use crate::dom::bindings::codegen::Bindings::XRBinding::{XRMethods, XRSessionMode};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::gamepad::Gamepad;
use crate::dom::gamepadevent::GamepadEventType;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::vrdisplay::VRDisplay;
use crate::dom::vrdisplayevent::VRDisplayEvent;
use crate::dom::xrsession::XRSession;
use dom_struct::dom_struct;
use ipc_channel::ipc::IpcSender;
use profile_traits::ipc;
use std::cell::Cell;
use std::rc::Rc;
use webvr_traits::{WebVRDisplayData, WebVRDisplayEvent, WebVREvent, WebVRMsg};
use webvr_traits::{WebVRGamepadData, WebVRGamepadEvent, WebVRGamepadState};

#[dom_struct]
pub struct XR {
    eventtarget: EventTarget,
    displays: DomRefCell<Vec<Dom<VRDisplay>>>,
    gamepads: DomRefCell<Vec<Dom<Gamepad>>>,
    pending_immersive_session: Cell<bool>,
    active_immersive_session: MutNullableDom<VRDisplay>,
}

impl XR {
    fn new_inherited() -> XR {
        XR {
            eventtarget: EventTarget::new_inherited(),
            displays: DomRefCell::new(Vec::new()),
            gamepads: DomRefCell::new(Vec::new()),
            pending_immersive_session: Cell::new(false),
            active_immersive_session: Default::default(),
        }
    }

    pub fn new(global: &GlobalScope) -> DomRoot<XR> {
        let root = reflect_dom_object(Box::new(XR::new_inherited()), global, XRBinding::Wrap);
        root.register();
        root
    }

    pub fn pending_or_active_session(&self) -> bool {
        self.pending_immersive_session.get() || self.active_immersive_session.get().is_some()
    }

    pub fn set_pending(&self) {
        self.pending_immersive_session.set(true)
    }

    pub fn set_active_immersive_session(&self, session: &VRDisplay) {
        // XXXManishearth when we support non-immersive (inline) sessions we should
        // ensure they never reach these codepaths
        self.pending_immersive_session.set(false);
        self.active_immersive_session.set(Some(session))
    }

    pub fn deactivate_session(&self) {
        self.pending_immersive_session.set(false);
        self.active_immersive_session.set(None)
    }
}

impl Drop for XR {
    fn drop(&mut self) {
        self.unregister();
    }
}

impl XRMethods for XR {
    /// https://immersive-web.github.io/webxr/#dom-xr-supportssessionmode
    fn SupportsSessionMode(&self, mode: XRSessionMode) -> Rc<Promise> {
        // XXXManishearth this should select an XR device first
        let promise = Promise::new(&self.global());
        if mode == XRSessionMode::Immersive_vr {
            promise.resolve_native(&());
        } else {
            // XXXManishearth support other modes
            promise.reject_error(Error::NotSupported);
        }

        promise
    }

    /// https://immersive-web.github.io/webxr/#dom-xr-requestsession
    fn RequestSession(&self, options: &XRSessionCreationOptions) -> Rc<Promise> {
        let promise = Promise::new(&self.global());
        if options.mode != XRSessionMode::Immersive_vr {
            promise.reject_error(Error::NotSupported);
            return promise;
        }

        if self.pending_or_active_session() {
            promise.reject_error(Error::InvalidState);
            return promise;
        }
        // we set pending immersive session to true further down
        // to handle rejections in a cleaner way

        let displays = self.get_displays();

        let displays = match displays {
            Ok(d) => d,
            Err(_) => {
                promise.reject_native(&());
                return promise;
            },
        };

        // XXXManishearth filter for displays which can_present
        if displays.is_empty() {
            promise.reject_error(Error::Security);
        }

        self.set_pending();

        let session = XRSession::new(&self.global(), &displays[0]);
        session.xr_present(promise.clone());
        promise
    }
}

impl XR {
    pub fn get_displays(&self) -> Result<Vec<DomRoot<VRDisplay>>, ()> {
        if let Some(webvr_thread) = self.webvr_thread() {
            let (sender, receiver) =
                ipc::channel(self.global().time_profiler_chan().clone()).unwrap();
            webvr_thread.send(WebVRMsg::GetDisplays(sender)).unwrap();

            // FIXME(#22505) we should not block here and instead produce a promise
            match receiver.recv().unwrap() {
                Ok(displays) => {
                    // Sync displays
                    for display in displays {
                        self.sync_display(&display);
                    }
                },
                Err(_) => return Err(()),
            }
        } else {
            // WebVR spec: The Promise MUST be rejected if WebVR is not enabled/supported.
            return Err(());
        }

        // convert from Dom to DomRoot
        Ok(self
            .displays
            .borrow()
            .iter()
            .map(|d| DomRoot::from_ref(&**d))
            .collect())
    }

    fn webvr_thread(&self) -> Option<IpcSender<WebVRMsg>> {
        self.global().as_window().webvr_thread()
    }

    fn find_display(&self, display_id: u32) -> Option<DomRoot<VRDisplay>> {
        self.displays
            .borrow()
            .iter()
            .find(|d| d.DisplayId() == display_id)
            .map(|d| DomRoot::from_ref(&**d))
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

    fn sync_display(&self, display: &WebVRDisplayData) -> DomRoot<VRDisplay> {
        if let Some(existing) = self.find_display(display.display_id) {
            existing.update_display(&display);
            existing
        } else {
            let root = VRDisplay::new(&self.global(), display.clone());
            self.displays.borrow_mut().push(Dom::from_ref(&*root));
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
            },
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
            },
        };
    }

    pub fn handle_webvr_event(&self, event: WebVREvent) {
        match event {
            WebVREvent::Display(event) => {
                self.handle_display_event(event);
            },
            WebVREvent::Gamepad(event) => {
                self.handle_gamepad_event(event);
            },
        };
    }

    pub fn handle_webvr_events(&self, events: Vec<WebVREvent>) {
        for event in events {
            self.handle_webvr_event(event);
        }
    }

    fn notify_display_event(&self, display: &VRDisplay, event: &WebVRDisplayEvent) {
        let event = VRDisplayEvent::new_from_webvr(&self.global(), &display, &event);
        event
            .upcast::<Event>()
            .fire(self.global().upcast::<EventTarget>());
    }
}

// Gamepad
impl XR {
    fn find_gamepad(&self, gamepad_id: u32) -> Option<DomRoot<Gamepad>> {
        self.gamepads
            .borrow()
            .iter()
            .find(|g| g.gamepad_id() == gamepad_id)
            .map(|g| DomRoot::from_ref(&**g))
    }

    fn sync_gamepad(&self, data: Option<WebVRGamepadData>, state: &WebVRGamepadState) {
        if let Some(existing) = self.find_gamepad(state.gamepad_id) {
            existing.update_from_vr(&state);
        } else {
            let index = self.gamepads.borrow().len();
            let data = data.unwrap_or_default();
            let root = Gamepad::new_from_vr(&self.global(), index as i32, &data, &state);
            self.gamepads.borrow_mut().push(Dom::from_ref(&*root));
            if state.connected {
                root.notify_event(GamepadEventType::Connected);
            }
        }
    }

    // Gamepads are synced immediately in response to the API call.
    // The current approach allows the to sample gamepad state multiple times per frame. This
    // guarantees that the gamepads always have a valid state and can be very useful for
    // motion capture or drawing applications.
    pub fn get_gamepads(&self) -> Vec<DomRoot<Gamepad>> {
        if let Some(wevbr_sender) = self.webvr_thread() {
            let (sender, receiver) =
                ipc::channel(self.global().time_profiler_chan().clone()).unwrap();
            let synced_ids = self
                .gamepads
                .borrow()
                .iter()
                .map(|g| g.gamepad_id())
                .collect();
            wevbr_sender
                .send(WebVRMsg::GetGamepads(synced_ids, sender))
                .unwrap();
            match receiver.recv().unwrap() {
                Ok(gamepads) => {
                    // Sync displays
                    for gamepad in gamepads {
                        self.sync_gamepad(gamepad.0, &gamepad.1);
                    }
                },
                Err(_) => {},
            }
        }

        // We can add other not VR related gamepad providers here
        self.gamepads
            .borrow()
            .iter()
            .map(|g| DomRoot::from_ref(&**g))
            .collect()
    }
}
