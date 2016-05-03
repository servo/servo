/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::HashChangeEventBinding;
use dom::bindings::codegen::Bindings::HashChangeEventBinding::HashChangeEventMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::URLBinding::URLMethods;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::USVString;
use dom::event::Event;
use dom::urlhelper::UrlHelper;
use string_cache::Atom;
use url::Url;
use util::str::DOMString;

// https://html.spec.whatwg.org/multipage/#hashchangeevent
#[dom_struct]
pub struct HashChangeEvent {
    event: Event,
    old_url: DOMRefCell<Option<Url>>,
    new_url: DOMRefCell<Option<Url>>,
}

impl HashChangeEvent {
    fn new_inherited() -> HashChangeEvent {
        HashChangeEvent {
            event: Event::new_inherited(),
            old_url: DOMRefCell::new(None),
            new_url: DOMRefCell::new(None),
        }
    }

    pub fn new_uninitialized(global: GlobalRef) -> Root<HashChangeEvent> {
        reflect_dom_object(box HashChangeEvent::new_inherited(),
                           global,
                           HashChangeEventBinding::Wrap)
    }

    pub fn new(global: GlobalRef,
               type_: Atom,
               bubbles: bool,
               cancelable: bool,
               old_url: Option<Url>,
               new_url: Option<Url>)
               -> Root<HashChangeEvent> {
        let ev = HashChangeEvent::new_uninitialized(global);
        *ev.old_url.borrow_mut() = old_url;
        *ev.new_url.borrow_mut() = new_url;
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev
    }

    #[allow(unsafe_code)]
    pub fn Constructor(global: GlobalRef,
                       type_: DOMString,
                       init: &HashChangeEventBinding::HashChangeEventInit)
                       -> Fallible<Root<HashChangeEvent>> {
        let old_url = init.oldURL.as_ref().and_then(|ref old_url| Url::parse(&old_url.0).ok());
        let new_url = init.newURL.as_ref().and_then(|ref new_url| Url::parse(&new_url.0).ok());
        Ok(HashChangeEvent::new(global,
                                Atom::from(type_),
                                init.parent.bubbles,
                                init.parent.cancelable,
                                old_url,
                                new_url))
    }
}

impl HashChangeEventMethods for HashChangeEvent {
    // https://html.spec.whatwg.org/multipage/#dom-hashchangeevent-oldurl
    fn OldURL(&self) -> USVString {
        match *self.old_url.borrow() {
            Some(ref old_url) => UrlHelper::Href(old_url),
            None => USVString(String::new()),
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-hashchangeevent-newurl
    fn NewURL(&self) -> USVString {
        match *self.new_url.borrow() {
            Some(ref new_url) => UrlHelper::Href(new_url),
            None => USVString(String::new()),
        }
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
