/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::HashChangeEventBinding;
use dom::bindings::codegen::Bindings::HashChangeEventBinding::HashChangeEventMethods;
use dom::bindings::codegen::Bindings::URLBinding::URLMethods;
use dom::bindings::error::{Error, Fallible};
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
    old_url: Url,
    new_url: Url,
}

impl HashChangeEvent {
    fn new_inherited(old_url: Url, new_url: Url) -> HashChangeEvent {
        HashChangeEvent {
            event: Event::new_inherited(),
            old_url: old_url,
            new_url: new_url,
        }
    }

    pub fn new_uninitialized(global: GlobalRef,
                             old_url: Url,
                             new_url: Url)
                             -> Root<HashChangeEvent> {
        reflect_dom_object(box HashChangeEvent::new_inherited(old_url, new_url),
                           global,
                           HashChangeEventBinding::Wrap)
    }

    pub fn new(global: GlobalRef,
               type_: Atom,
               bubbles: bool,
               cancelable: bool,
               old_url: Url,
               new_url: Url)
               -> Root<HashChangeEvent> {
        let ev = HashChangeEvent::new_uninitialized(global, old_url, new_url);
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
        let old_url = match Url::parse(&init.oldURL.0) {
            Ok(old_url) => old_url,
            Err(error) => return Err(Error::Type(format!("could not parse URL: {}", error))),
        };
        let new_url = match Url::parse(&init.newURL.0) {
            Ok(new_url) => new_url,
            Err(error) => return Err(Error::Type(format!("could not parse URL: {}", error))),
        };
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
        UrlHelper::Href(&self.old_url)
    }

    // https://html.spec.whatwg.org/multipage/#dom-hashchangeevent-newurl
    fn NewURL(&self) -> USVString {
        UrlHelper::Href(&self.new_url)
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}
