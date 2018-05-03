/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::MessageEventBinding;
use dom::bindings::codegen::Bindings::MessageEventBinding::MessageEventMethods;
use dom::bindings::error::Fallible;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::bindings::trace::RootedTraceableBox;
use dom::event::Event;
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use js::jsapi::{Heap, JSContext};
use js::jsval::JSVal;
use js::rust::HandleValue;
use servo_atoms::Atom;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct MessageEvent<TH: TypeHolderTrait> {
    event: Event<TH>,
    data: Heap<JSVal>,
    origin: DOMString,
    lastEventId: DOMString,
}

impl<TH: TypeHolderTrait> MessageEvent<TH> {
    pub fn new_uninitialized(global: &GlobalScope<TH>) -> DomRoot<MessageEvent<TH>> {
        MessageEvent::new_initialized(global,
                                      HandleValue::undefined(),
                                      DOMString::new(),
                                      DOMString::new())
    }

    pub fn new_initialized(global: &GlobalScope<TH>,
                           data: HandleValue,
                           origin: DOMString,
                           lastEventId: DOMString) -> DomRoot<MessageEvent<TH>> {
        let ev = Box::new(MessageEvent {
            event: Event::new_inherited(),
            data: Heap::default(),
            origin: origin,
            lastEventId: lastEventId,
        });
        let ev = reflect_dom_object(ev, global, MessageEventBinding::Wrap);
        ev.data.set(data.get());

        ev
    }

    pub fn new(global: &GlobalScope<TH>, type_: Atom,
               bubbles: bool, cancelable: bool,
               data: HandleValue, origin: DOMString, lastEventId: DOMString)
               -> DomRoot<MessageEvent<TH>> {
        let ev = MessageEvent::new_initialized(global, data, origin, lastEventId);
        {
            let event = ev.upcast::<Event<TH>>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev
    }

    pub fn Constructor(global: &GlobalScope<TH>,
                       type_: DOMString,
                       init: RootedTraceableBox<MessageEventBinding::MessageEventInit>)
                       -> Fallible<DomRoot<MessageEvent<TH>>> {
        let ev = MessageEvent::new(global,
                                   Atom::from(type_),
                                   init.parent.bubbles,
                                   init.parent.cancelable,
                                   init.data.handle(),
                                   init.origin.clone(),
                                   init.lastEventId.clone());
        Ok(ev)
    }
}

impl<TH: TypeHolderTrait> MessageEvent<TH> {
    pub fn dispatch_jsval(target: &EventTarget<TH>,
                          scope: &GlobalScope<TH>,
                          message: HandleValue) {
        let messageevent = MessageEvent::new(
            scope,
            atom!("message"),
            false,
            false,
            message,
            DOMString::new(),
            DOMString::new());
        messageevent.upcast::<Event<TH>>().fire(target);
    }
}

impl<TH: TypeHolderTrait> MessageEventMethods for MessageEvent<TH> {
    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-messageevent-data
    unsafe fn Data(&self, _cx: *mut JSContext) -> JSVal {
        self.data.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-messageevent-origin
    fn Origin(&self) -> DOMString {
        self.origin.clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-messageevent-lasteventid
    fn LastEventId(&self) -> DOMString {
        self.lastEventId.clone()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
