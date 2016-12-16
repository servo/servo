/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::VRDisplayEventBinding;
use dom::bindings::codegen::Bindings::VRDisplayEventBinding::VRDisplayEventMethods;
use dom::bindings::codegen::Bindings::VRDisplayEventBinding::VRDisplayEventReason;
use dom::bindings::error::Fallible;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::event::Event;
use dom::globalscope::GlobalScope;
use dom::vrdisplay::VRDisplay;
use dom::window::Window;
use servo_atoms::Atom;
use vr_traits::webvr;

#[dom_struct]
pub struct VRDisplayEvent {
    event: Event,
    display: JS<VRDisplay>,
    reason: Option<VRDisplayEventReason>
}

impl VRDisplayEvent {
    fn new_inherited(display: &VRDisplay,
                     reason: &Option<VRDisplayEventReason>)
                     -> VRDisplayEvent {
        VRDisplayEvent {
            event: Event::new_inherited(),
            display: JS::from_ref(display),
            reason: reason.clone()
        }
    }

    pub fn new(global: &GlobalScope,
               type_: Atom,
               bubbles: bool,
               cancelable: bool,
               display: &VRDisplay,
               reason: &Option<VRDisplayEventReason>)
               -> Root<VRDisplayEvent> {
        let ev = reflect_dom_object(box VRDisplayEvent::new_inherited(&display, reason),
                           global,
                           VRDisplayEventBinding::Wrap);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev
    }

    pub fn new_from_webvr(global: &GlobalScope,
                          display: &VRDisplay,
                          event: &webvr::VRDisplayEvent)
                          -> Root<VRDisplayEvent> {
        let (name, reason) = match event {
            &webvr::VRDisplayEvent::Connect(_) => ("displayconnect", None),
            &webvr::VRDisplayEvent::Disconnect(_) => ("displaydisconnect", None),
            &webvr::VRDisplayEvent::Activate(_, ref reason) => ("activate", Some(reason)),
            &webvr::VRDisplayEvent::Deactivate(_, ref reason) => ("deactivate", Some(reason)),
            &webvr::VRDisplayEvent::Blur(_) => ("blur", None),
            &webvr::VRDisplayEvent::Focus(_) => ("focus", None),
            &webvr::VRDisplayEvent::PresentChange(_, _) => ("presentchange", None),
            &webvr::VRDisplayEvent::Change(_) => panic!("VRDisplayEvent:Change event not available in WebVR")
        };

        // map to JS enum values
        let reason = reason.map(|r| {
            match r {
                &webvr::VRDisplayEventReason::Navigation => VRDisplayEventReason::Navigation,
                &webvr::VRDisplayEventReason::Mounted => VRDisplayEventReason::Mounted,
                &webvr::VRDisplayEventReason::Unmounted => VRDisplayEventReason::Unmounted,
            }
        });

        VRDisplayEvent::new(&global,
                            Atom::from(DOMString::from(name)),
                            false,
                            false,
                            &display,
                            &reason)
    }

    pub fn Constructor(window: &Window,
                       type_: DOMString,
                       init: &VRDisplayEventBinding::VRDisplayEventInit)
                       -> Fallible<Root<VRDisplayEvent>> {
        Ok(VRDisplayEvent::new(&window.global(),
                            Atom::from(type_),
                            init.parent.bubbles,
                            init.parent.cancelable,
                            &init.display,
                            &init.reason))
    }
}

impl VRDisplayEventMethods for VRDisplayEvent {
    // https://w3c.github.io/webvr/#dom-vrdisplayevent-display
    fn Display(&self) -> Root<VRDisplay> {
        Root::from_ref(&*self.display)
    }

    // https://w3c.github.io/webvr/#enumdef-vrdisplayeventreason
    fn GetReason(&self) -> Option<VRDisplayEventReason> {
        self.reason
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
