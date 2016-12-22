/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::MediaQueryListEventBinding;
use dom::bindings::codegen::Bindings::MediaQueryListEventBinding::MediaQueryListEventInit;
use dom::bindings::codegen::Bindings::MediaQueryListEventBinding::MediaQueryListEventMethods;
use dom::bindings::error::Fallible;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::DOMString;
use dom::event::Event;
use dom::globalscope::GlobalScope;
use dom::window::Window;
use servo_atoms::Atom;
use std::cell::Cell;

// https://drafts.csswg.org/cssom-view/#dom-mediaquerylistevent-mediaquerylistevent
#[dom_struct]
pub struct MediaQueryListEvent {
    event: Event,
    media: DOMString,
    matches: Cell<bool>
}

impl MediaQueryListEvent {
    pub fn new_initialized(global: &GlobalScope,
                           media: DOMString,
                           matches: bool) -> Root<MediaQueryListEvent> {
        let ev = box MediaQueryListEvent {
            event: Event::new_inherited(),
            media: media,
            matches: Cell::new(matches)
        };
        reflect_dom_object(ev, global, MediaQueryListEventBinding::Wrap)
    }

    pub fn new(global: &GlobalScope, type_: Atom,
               bubbles: bool, cancelable: bool,
               media: DOMString, matches: bool) -> Root<MediaQueryListEvent> {
        let ev = MediaQueryListEvent::new_initialized(global, media, matches);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev
    }

    pub fn Constructor(window: &Window, type_: DOMString,
                       init: &MediaQueryListEventInit)
                       -> Fallible<Root<MediaQueryListEvent>> {
        let global = window.upcast::<GlobalScope>();
        Ok(MediaQueryListEvent::new(global, Atom::from(type_),
                                    init.parent.bubbles, init.parent.cancelable,
                                    init.media.clone(), init.matches))
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
