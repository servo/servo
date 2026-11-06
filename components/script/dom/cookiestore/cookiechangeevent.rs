/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::jsapi::Heap;
use js::jsval::JSVal;
use js::rust::{HandleObject, MutableHandleValue};
use script_bindings::reflector::reflect_dom_object_with_proto;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::CookieChangeEventBinding::{
    CookieChangeEventInit, CookieChangeEventMethods,
};
use crate::dom::bindings::codegen::Bindings::CookieStoreBinding::CookieListItem;
use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::utils::to_frozen_array;
use crate::dom::event::Event;
use crate::dom::window::Window;
use crate::realms::enter_auto_realm;

/// <https://cookiestore.spec.whatwg.org/#cookiechangeevent>
#[dom_struct]
pub(crate) struct CookieChangeEvent {
    event: Event,
    #[ignore_malloc_size_of = "mozjs"]
    changed: Heap<JSVal>,
    #[ignore_malloc_size_of = "mozjs"]
    deleted: Heap<JSVal>,
}

impl CookieChangeEvent {
    #[expect(clippy::too_many_arguments)]
    pub(crate) fn new_with_proto(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        changed: &[CookieListItem],
        deleted: &[CookieListItem],
    ) -> DomRoot<CookieChangeEvent> {
        let ev = reflect_dom_object_with_proto(
            cx,
            Box::new(CookieChangeEvent {
                event: Event::new_inherited(),
                changed: Heap::default(),
                deleted: Heap::default(),
            }),
            window,
            proto,
        );
        ev.upcast::<Event>().init_event(type_, bubbles, cancelable);
        let mut realm = enter_auto_realm(cx, window);
        let cx = &mut realm.current_realm();
        rooted!(&in(cx) let mut frozen_val: JSVal);
        to_frozen_array(cx, changed, frozen_val.handle_mut());
        ev.changed.set(*frozen_val);
        to_frozen_array(cx, deleted, frozen_val.handle_mut());
        ev.deleted.set(*frozen_val);
        ev
    }
}

impl CookieChangeEventMethods<crate::DomTypeHolder> for CookieChangeEvent {
    /// <https://cookiestore.spec.whatwg.org/#dom-cookiechangeevent-cookiechangeevent>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &CookieChangeEventInit,
    ) -> DomRoot<CookieChangeEvent> {
        CookieChangeEvent::new_with_proto(
            cx,
            window,
            proto,
            Atom::from(type_),
            init.parent.bubbles,
            init.parent.cancelable,
            init.changed.as_deref().unwrap_or_default(),
            init.deleted.as_deref().unwrap_or_default(),
        )
    }

    /// <https://cookiestore.spec.whatwg.org/#dom-cookiechangeevent-changed>
    fn Changed(&self, mut retval: MutableHandleValue) {
        // The changed and deleted attributes must return the value they were initialized to.
        retval.set(self.changed.get());
    }

    /// <https://cookiestore.spec.whatwg.org/#dom-cookiechangeevent-deleted>
    fn Deleted(&self, mut retval: MutableHandleValue) {
        // The changed and deleted attributes must return the value they were initialized to.
        retval.set(self.deleted.get());
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
