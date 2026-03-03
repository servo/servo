use dom_struct::dom_struct;
use js::gc::{HandleObject, MutableHandleValue};
use stylo_atoms::Atom;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::CookieChangeEventBinding::{
    CookieChangeEventInit, CookieChangeEventMethods,
};
use crate::dom::bindings::codegen::Bindings::CookieStoreBinding::CookieListItem;
use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::frozenarray::CachedFrozenArray;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::utils::to_frozen_array;
use crate::dom::event::Event;
use crate::dom::window::Window;
use crate::script_runtime::{CanGc, JSContext};

#[dom_struct]
pub(crate) struct CookieChangeEvent {
    event: Event,
    changed: DomRefCell<Option<Vec<CookieListItem>>>,

    deleted: DomRefCell<Option<Vec<CookieListItem>>>,

}

impl CookieChangeEvent {
    fn new_inherited(init: &CookieChangeEventInit) -> CookieChangeEvent {
        CookieChangeEvent {
            event: Event::new_inherited(),
            changed: DomRefCell::new(init.changed.clone()),
            deleted: DomRefCell::new(init.deleted.clone()),
        }
    }

    pub(crate) fn new(
        window: &Window,
        type_: Atom,
        init: &CookieChangeEventInit,
        can_gc: CanGc,
    ) -> DomRoot<CookieChangeEvent> {
        Self::new_with_proto(window, None, type_, init, can_gc)
    }

    fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        type_: Atom,
        init: &CookieChangeEventInit,
        can_gc: CanGc,
    ) -> DomRoot<CookieChangeEvent> {
        let event = reflect_dom_object_with_proto(
            Box::new(CookieChangeEvent::new_inherited(init)),
            window,
            proto,
            can_gc,
        );
        {
            let event = event.upcast::<Event>();
            event.init_event(type_, init.parent.bubbles, init.parent.cancelable);
        }
        event
    }
}

impl CookieChangeEventMethods<crate::DomTypeHolder> for CookieChangeEvent {
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        type_: DOMString,
        init: &CookieChangeEventInit,
    ) -> DomRoot<CookieChangeEvent> {
        CookieChangeEvent::new_with_proto(window, proto, Atom::from(type_), init, can_gc)
    }

    fn Changed(&self, cx: JSContext, can_gc: CanGc, retval: MutableHandleValue) {
        to_frozen_array(self.changed.borrow().as_slice(), cx, retval, can_gc)
    }

    fn Deleted(&self, cx: JSContext, can_gc: CanGc, retval: MutableHandleValue) {
        to_frozen_array(self.deleted.borrow().as_slice(), cx, retval, can_gc)
    }

    fn IsTrusted(&self) -> bool {
        self.upcast::<Event>().IsTrusted()
    }
}
