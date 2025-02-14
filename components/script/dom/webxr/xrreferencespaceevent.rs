/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::codegen::Bindings::XRReferenceSpaceEventBinding::{
    XRReferenceSpaceEventInit, XRReferenceSpaceEventMethods,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;
use crate::dom::xrreferencespace::XRReferenceSpace;
use crate::dom::xrrigidtransform::XRRigidTransform;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct XRReferenceSpaceEvent {
    event: Event,
    space: Dom<XRReferenceSpace>,
    transform: Option<Dom<XRRigidTransform>>,
}

impl XRReferenceSpaceEvent {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_inherited(
        space: &XRReferenceSpace,
        transform: Option<&XRRigidTransform>,
    ) -> XRReferenceSpaceEvent {
        XRReferenceSpaceEvent {
            event: Event::new_inherited(),
            space: Dom::from_ref(space),
            transform: transform.map(Dom::from_ref),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        space: &XRReferenceSpace,
        transform: Option<&XRRigidTransform>,
        can_gc: CanGc,
    ) -> DomRoot<XRReferenceSpaceEvent> {
        Self::new_with_proto(
            global, None, type_, bubbles, cancelable, space, transform, can_gc,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        space: &XRReferenceSpace,
        transform: Option<&XRRigidTransform>,
        can_gc: CanGc,
    ) -> DomRoot<XRReferenceSpaceEvent> {
        let trackevent = reflect_dom_object_with_proto(
            Box::new(XRReferenceSpaceEvent::new_inherited(space, transform)),
            global,
            proto,
            can_gc,
        );
        {
            let event = trackevent.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        trackevent
    }
}

impl XRReferenceSpaceEventMethods<crate::DomTypeHolder> for XRReferenceSpaceEvent {
    /// <https://www.w3.org/TR/webxr/#dom-xrreferencespaceevent-xrreferencespaceevent>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        type_: DOMString,
        init: &XRReferenceSpaceEventInit,
    ) -> Fallible<DomRoot<XRReferenceSpaceEvent>> {
        Ok(XRReferenceSpaceEvent::new_with_proto(
            &window.global(),
            proto,
            Atom::from(type_),
            init.parent.bubbles,
            init.parent.cancelable,
            &init.referenceSpace,
            init.transform.as_deref(),
            can_gc,
        ))
    }

    /// <https://www.w3.org/TR/webxr/#dom-xrreferencespaceeventinit-session>
    fn ReferenceSpace(&self) -> DomRoot<XRReferenceSpace> {
        DomRoot::from_ref(&*self.space)
    }

    /// <https://www.w3.org/TR/webxr/#dom-xrreferencespaceevent-transform>
    fn GetTransform(&self) -> Option<DomRoot<XRRigidTransform>> {
        self.transform
            .as_ref()
            .map(|transform| DomRoot::from_ref(&**transform))
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
