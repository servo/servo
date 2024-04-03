/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::conversions::ToJSValConvertible;
use js::jsapi::Heap;
use js::jsval::{JSVal, UndefinedValue};
use js::rust::HandleObject;
use servo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::codegen::Bindings::XRInputSourcesChangeEventBinding::{
    self, XRInputSourcesChangeEventMethods,
};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;
use crate::dom::xrinputsource::XRInputSource;
use crate::dom::xrsession::XRSession;
use crate::realms::enter_realm;
use crate::script_runtime::JSContext;

#[dom_struct]
pub struct XRInputSourcesChangeEvent {
    event: Event,
    session: Dom<XRSession>,
    #[ignore_malloc_size_of = "mozjs"]
    added: Heap<JSVal>,
    #[ignore_malloc_size_of = "mozjs"]
    removed: Heap<JSVal>,
}

impl XRInputSourcesChangeEvent {
    #[allow(crown::unrooted_must_root)]
    fn new_inherited(session: &XRSession) -> XRInputSourcesChangeEvent {
        XRInputSourcesChangeEvent {
            event: Event::new_inherited(),
            session: Dom::from_ref(session),
            added: Heap::default(),
            removed: Heap::default(),
        }
    }

    pub fn new(
        global: &GlobalScope,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        session: &XRSession,
        added: &[DomRoot<XRInputSource>],
        removed: &[DomRoot<XRInputSource>],
    ) -> DomRoot<XRInputSourcesChangeEvent> {
        Self::new_with_proto(
            global, None, type_, bubbles, cancelable, session, added, removed,
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
    ) -> DomRoot<XRInputSourcesChangeEvent> {
        let changeevent = reflect_dom_object_with_proto(
            Box::new(XRInputSourcesChangeEvent::new_inherited(session)),
            global,
            proto,
        );
        {
            let event = changeevent.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        let _ac = enter_realm(global);
        let cx = GlobalScope::get_cx();
        unsafe {
            rooted!(in(*cx) let mut added_val = UndefinedValue());
            added.to_jsval(*cx, added_val.handle_mut());
            changeevent.added.set(added_val.get());
            rooted!(in(*cx) let mut removed_val = UndefinedValue());
            removed.to_jsval(*cx, removed_val.handle_mut());
            changeevent.removed.set(removed_val.get());
        }

        changeevent
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
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
        )
    }
}

impl XRInputSourcesChangeEventMethods for XRInputSourcesChangeEvent {
    // https://immersive-web.github.io/webxr/#dom-xrinputsourceschangeevent-session
    fn Session(&self) -> DomRoot<XRSession> {
        DomRoot::from_ref(&*self.session)
    }

    // https://immersive-web.github.io/webxr/#dom-xrinputsourceschangeevent-added
    fn Added(&self, _cx: JSContext) -> JSVal {
        self.added.get()
    }

    // https://immersive-web.github.io/webxr/#dom-xrinputsourceschangeevent-removed
    fn Removed(&self, _cx: JSContext) -> JSVal {
        self.removed.get()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
