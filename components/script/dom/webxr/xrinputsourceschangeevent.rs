/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::jsapi::Heap;
use js::jsval::JSVal;
use js::rust::{HandleObject, MutableHandleValue};
use servo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::codegen::Bindings::XRInputSourcesChangeEventBinding::{
    self, XRInputSourcesChangeEventMethods,
};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::utils::to_frozen_array;
use crate::dom::event::Event;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;
use crate::dom::xrinputsource::XRInputSource;
use crate::dom::xrsession::XRSession;
use crate::realms::enter_realm;
use crate::script_runtime::{CanGc, JSContext};

#[dom_struct]
pub(crate) struct XRInputSourcesChangeEvent {
    event: Event,
    session: Dom<XRSession>,
    #[ignore_malloc_size_of = "mozjs"]
    added: Heap<JSVal>,
    #[ignore_malloc_size_of = "mozjs"]
    removed: Heap<JSVal>,
}

impl XRInputSourcesChangeEvent {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_inherited(session: &XRSession) -> XRInputSourcesChangeEvent {
        XRInputSourcesChangeEvent {
            event: Event::new_inherited(),
            session: Dom::from_ref(session),
            added: Heap::default(),
            removed: Heap::default(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        global: &GlobalScope,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        session: &XRSession,
        added: &[DomRoot<XRInputSource>],
        removed: &[DomRoot<XRInputSource>],
        can_gc: CanGc,
    ) -> DomRoot<XRInputSourcesChangeEvent> {
        Self::new_with_proto(
            global, None, type_, bubbles, cancelable, session, added, removed, can_gc,
        )
    }

    #[allow(unsafe_code)]
    #[allow(clippy::too_many_arguments)]
    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        session: &XRSession,
        added: &[DomRoot<XRInputSource>],
        removed: &[DomRoot<XRInputSource>],
        can_gc: CanGc,
    ) -> DomRoot<XRInputSourcesChangeEvent> {
        let changeevent = reflect_dom_object_with_proto(
            Box::new(XRInputSourcesChangeEvent::new_inherited(session)),
            global,
            proto,
            can_gc,
        );
        {
            let event = changeevent.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        let _ac = enter_realm(global);
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut frozen_val: JSVal);
        to_frozen_array(added, cx, frozen_val.handle_mut());
        changeevent.added.set(*frozen_val);
        to_frozen_array(removed, cx, frozen_val.handle_mut());
        changeevent.removed.set(*frozen_val);
        changeevent
    }
}

impl XRInputSourcesChangeEventMethods<crate::DomTypeHolder> for XRInputSourcesChangeEvent {
    // https://immersive-web.github.io/webxr/#dom-xrinputsourceschangeevent-xrinputsourceschangeevent
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        type_: DOMString,
        init: &XRInputSourcesChangeEventBinding::XRInputSourcesChangeEventInit,
    ) -> DomRoot<XRInputSourcesChangeEvent> {
        XRInputSourcesChangeEvent::new_with_proto(
            &window.global(),
            proto,
            Atom::from(type_),
            init.parent.bubbles,
            init.parent.cancelable,
            &init.session,
            &init.added,
            &init.removed,
            can_gc,
        )
    }

    // https://immersive-web.github.io/webxr/#dom-xrinputsourceschangeevent-session
    fn Session(&self) -> DomRoot<XRSession> {
        DomRoot::from_ref(&*self.session)
    }

    // https://immersive-web.github.io/webxr/#dom-xrinputsourceschangeevent-added
    fn Added(&self, _cx: JSContext, mut retval: MutableHandleValue) {
        retval.set(self.added.get())
    }

    // https://immersive-web.github.io/webxr/#dom-xrinputsourceschangeevent-removed
    fn Removed(&self, _cx: JSContext, mut retval: MutableHandleValue) {
        retval.set(self.removed.get())
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
