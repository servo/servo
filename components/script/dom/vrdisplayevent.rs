/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::VRDisplayEventBinding;
use dom::bindings::codegen::Bindings::VRDisplayEventBinding::VRDisplayEventMethods;
use dom::bindings::codegen::Bindings::VRDisplayEventBinding::VRDisplayEventReason;
use dom::bindings::error::Fallible;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot};
use dom::bindings::str::DOMString;
use dom::event::Event;
use dom::globalscope::GlobalScope;
use dom::vrdisplay::VRDisplay;
use dom::window::Window;
use dom_struct::dom_struct;
use servo_atoms::Atom;
use typeholder::TypeHolderTrait;
use webvr_traits::{WebVRDisplayEvent, WebVRDisplayEventReason};

#[dom_struct]
pub struct VRDisplayEvent<TH: TypeHolderTrait> {
    event: Event<TH>,
    display: Dom<VRDisplay<TH>>,
    reason: Option<VRDisplayEventReason>
}

impl<TH: TypeHolderTrait> VRDisplayEvent<TH> {
    fn new_inherited(display: &VRDisplay<TH>,
                     reason: Option<VRDisplayEventReason>)
                     -> VRDisplayEvent<TH> {
        VRDisplayEvent {
            event: Event::new_inherited(),
            display: Dom::from_ref(display),
            reason: reason.clone()
        }
    }

    pub fn new(global: &GlobalScope<TH>,
               type_: Atom,
               bubbles: bool,
               cancelable: bool,
               display: &VRDisplay<TH>,
               reason: Option<VRDisplayEventReason>)
               -> DomRoot<VRDisplayEvent<TH>> {
        let ev = reflect_dom_object(
            Box::new(VRDisplayEvent::new_inherited(&display, reason)),
            global,
            VRDisplayEventBinding::Wrap
        );
        {
            let event = ev.upcast::<Event<TH>>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev
    }

    pub fn new_from_webvr(global: &GlobalScope<TH>,
                          display: &VRDisplay<TH>,
                          event: &WebVRDisplayEvent)
                          -> DomRoot<VRDisplayEvent<TH>> {
        let (name, reason) = match *event {
            WebVRDisplayEvent::Connect(_) => ("vrdisplayconnect", None),
            WebVRDisplayEvent::Disconnect(_) => ("vrdisplaydisconnect", None),
            WebVRDisplayEvent::Activate(_, reason) => ("vrdisplayactivate", Some(reason)),
            WebVRDisplayEvent::Deactivate(_, reason) => ("vrdisplaydeactivate", Some(reason)),
            WebVRDisplayEvent::Blur(_) => ("vrdisplayblur", None),
            WebVRDisplayEvent::Focus(_) => ("vrdisplayfocus", None),
            WebVRDisplayEvent::PresentChange(_, _) => ("vrdisplaypresentchange", None),
            WebVRDisplayEvent::Change(_) |
            WebVRDisplayEvent::Pause(_) |
            WebVRDisplayEvent::Resume(_) |
            WebVRDisplayEvent::Exit(_) => {
                panic!("{:?} event not available in WebVR", event)
            }
        };

        // map to JS enum values
        let reason = reason.map(|r| {
            match r {
                WebVRDisplayEventReason::Navigation => VRDisplayEventReason::Navigation,
                WebVRDisplayEventReason::Mounted => VRDisplayEventReason::Mounted,
                WebVRDisplayEventReason::Unmounted => VRDisplayEventReason::Unmounted,
            }
        });

        VRDisplayEvent::new(&global,
                            Atom::from(DOMString::from(name)),
                            false,
                            false,
                            &display,
                            reason)
    }

    pub fn Constructor(window: &Window<TH>,
                       type_: DOMString,
                       init: &VRDisplayEventBinding::VRDisplayEventInit<TH>)
                       -> Fallible<DomRoot<VRDisplayEvent<TH>>> {
        Ok(VRDisplayEvent::new(&window.global(),
                            Atom::from(type_),
                            init.parent.bubbles,
                            init.parent.cancelable,
                            &init.display,
                            init.reason))
    }
}

impl<TH: TypeHolderTrait> VRDisplayEventMethods<TH> for VRDisplayEvent<TH> {
    // https://w3c.github.io/webvr/#dom-vrdisplayevent-display
    fn Display(&self) -> DomRoot<VRDisplay<TH>> {
        DomRoot::from_ref(&*self.display)
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
