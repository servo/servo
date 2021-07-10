/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::MediaQueryListEventBinding::MediaQueryListEventInit;
use crate::dom::bindings::codegen::Bindings::MediaQueryListEventBinding::MediaQueryListEventMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::event::Event;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;
use dom_struct::dom_struct;
use servo_atoms::Atom;
use std::cell::Cell;

// https://drafts.csswg.org/cssom-view/#dom-mediaquerylistevent-mediaquerylistevent
#[dom_struct]
pub struct MediaQueryListEvent {
    event: Event,
    media: DOMString,
    matches: Cell<bool>,
}

impl MediaQueryListEvent {
    pub fn new_initialized(
        global: &GlobalScope,
        media: DOMString,
        matches: bool,
    ) -> DomRoot<MediaQueryListEvent> {
        let ev = Box::new(MediaQueryListEvent {
            event: Event::new_inherited(),
            media: media,
            matches: Cell::new(matches),
        });
        reflect_dom_object(ev, global)
    }

    pub fn new(
        global: &GlobalScope,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        media: DOMString,
        matches: bool,
    ) -> DomRoot<MediaQueryListEvent> {
        let ev = MediaQueryListEvent::new_initialized(global, media, matches);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        window: &Window,
        type_: DOMString,
        init: &MediaQueryListEventInit,
    ) -> Fallible<DomRoot<MediaQueryListEvent>> {
        let global = window.upcast::<GlobalScope>();
        Ok(MediaQueryListEvent::new(
            global,
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
