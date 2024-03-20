/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::MediaQueryListEventBinding::{
    MediaQueryListEventInit, MediaQueryListEventMethods,
};
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;

// https://drafts.csswg.org/cssom-view/#dom-mediaquerylistevent-mediaquerylistevent
#[dom_struct]
pub struct MediaQueryListEvent {
    event: Event,
    media: DOMString,
    matches: Cell<bool>,
}

impl MediaQueryListEvent {
    fn new_initialized(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        media: DOMString,
        matches: bool,
    ) -> DomRoot<MediaQueryListEvent> {
        let ev = Box::new(MediaQueryListEvent {
            event: Event::new_inherited(),
            media,
            matches: Cell::new(matches),
        });
        reflect_dom_object_with_proto(ev, global, proto)
    }

    pub fn new(
        global: &GlobalScope,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        media: DOMString,
        matches: bool,
    ) -> DomRoot<MediaQueryListEvent> {
        Self::new_with_proto(global, None, type_, bubbles, cancelable, media, matches)
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        media: DOMString,
        matches: bool,
    ) -> DomRoot<MediaQueryListEvent> {
        let ev = MediaQueryListEvent::new_initialized(global, proto, media, matches);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &MediaQueryListEventInit,
    ) -> Fallible<DomRoot<MediaQueryListEvent>> {
        let global = window.upcast::<GlobalScope>();
        Ok(MediaQueryListEvent::new_with_proto(
            global,
            proto,
            Atom::from(type_),
            init.parent.bubbles,
            init.parent.cancelable,
            init.media.clone(),
            init.matches,
        ))
    }
}

impl MediaQueryListEventMethods for MediaQueryListEvent {
    // https://drafts.csswg.org/cssom-view/#dom-mediaquerylistevent-media
    fn Media(&self) -> DOMString {
        self.media.clone()
    }

    // https://drafts.csswg.org/cssom-view/#dom-mediaquerylistevent-matches
    fn Matches(&self) -> bool {
        self.matches.get()
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.upcast::<Event>().IsTrusted()
    }
}
