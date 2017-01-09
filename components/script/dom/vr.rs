/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::VRBinding;
use dom::bindings::codegen::Bindings::VRBinding::VRMethods;
use dom::bindings::error::Error;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::event::Event;
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;
use dom::promise::Promise;
use dom::vrdisplay::VRDisplay;
use dom::vrdisplayevent::VRDisplayEvent;
use ipc_channel::ipc;
use ipc_channel::ipc::IpcSender;
use script_traits::WebVREventMsg;
use std::rc::Rc;
use webvr_traits::WebVRMsg;
use webvr_traits::webvr;

#[dom_struct]
pub struct VR {
    eventtarget: EventTarget,
    displays: DOMRefCell<Vec<JS<VRDisplay>>>
}

impl VR {
    fn new_inherited() -> VR {
        VR {
            eventtarget: EventTarget::new_inherited(),
            displays: DOMRefCell::new(Vec::new())
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

    fn find_display(&self, display_id: u64) -> Option<Root<VRDisplay>> {
        self.displays.borrow()
                     .iter()
                     .find(|d| d.get_display_id() == display_id)
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

    fn sync_display(&self, display: &webvr::VRDisplayData) -> Root<VRDisplay> {
        if let Some(existing) = self.find_display(display.display_id) {
            existing.update_display(&display);
            existing
        } else {
            let root = VRDisplay::new(&self.global(), display.clone());
            self.displays.borrow_mut().push(JS::from_ref(&*root));
            root
        }
    }

    pub fn handle_webvr_event(&self, event: WebVREventMsg) {
        let WebVREventMsg::DisplayEvent(event) = event;
        match &event {
            &webvr::VRDisplayEvent::Connect(ref display) => {
                let display = self.sync_display(&display);
                display.handle_webvr_event(&event);
                self.notify_event(&display, &event);
            },
            &webvr::VRDisplayEvent::Disconnect(id) => {
                if let Some(display) = self.find_display(id) {
                    display.handle_webvr_event(&event);
                    self.notify_event(&display, &event);
                }
            },
            &webvr::VRDisplayEvent::Activate(ref display, _) |
            &webvr::VRDisplayEvent::Deactivate(ref display, _) |
            &webvr::VRDisplayEvent::Blur(ref display) |
            &webvr::VRDisplayEvent::Focus(ref display) |
            &webvr::VRDisplayEvent::PresentChange(ref display, _) |
            &webvr::VRDisplayEvent::Change(ref display) => {
                let display = self.sync_display(&display);
                display.handle_webvr_event(&event);
            }
        };
    }

    fn notify_event(&self, display: &VRDisplay, event: &webvr::VRDisplayEvent) {
        let event = VRDisplayEvent::new_from_webvr(&self.global(), &display, &event);
        event.upcast::<Event>().fire(self.upcast());
    }
}

