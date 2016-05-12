/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HistoryBinding::{self, HistoryMethods, ScrollRestoration};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::structuredclone::StructuredCloneData;
use dom::window::Window;
use js::jsapi::{JSContext, HandleValue};
use js::jsval::JSVal;
use util::str::DOMString;

// https://html.spec.whatwg.org/multipage/#the-history-interface
#[dom_struct]
pub struct History {
    reflector_: Reflector,
    window: JS<Window>,
}

impl History {
    pub fn new_inherited(window: &Window) -> History {
        History {
            reflector_: Reflector::new(),
            window: JS::from_ref(&window),
        }
    }

    pub fn new(window: &Window) -> Root<History> {
        reflect_dom_object(box History::new_inherited(window),
                           GlobalRef::Window(window),
                           HistoryBinding::Wrap)
    }
}

impl HistoryMethods for History {
    // https://html.spec.whatwg.org/multipage/#dom-history-length
    fn Length(&self) -> u32 {
        // TODO: Consider how this should be casted
        self.window.browsing_context().session_history_length() as u32
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-scroll-restoration
    fn ScrollRestoration(&self) -> ScrollRestoration {
        ScrollRestoration::Auto
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-scroll-restoration
    fn SetScrollRestoration(&self, value: ScrollRestoration) {

    }

    // https://html.spec.whatwg.org/multipage/#dom-history-state
    fn State(&self, _cx: *mut JSContext) -> JSVal {
        self.window.browsing_context().state()
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-go
    fn Go(&self, delta: i32) {
        self.window.browsing_context().go(delta)
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-back
    fn Back(&self) {
        self.window.browsing_context().go(-1)
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-forward
    fn Forward(&self) {
        self.window.browsing_context().go(1)
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-pushstate
    fn PushState(&self, cx: *mut JSContext, data: HandleValue, title: DOMString, url: Option<DOMString>) {
        let data = StructuredCloneData::write(cx, data).unwrap();
        self.window.browsing_context().push_state(data, title, url);
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-replacestate
    fn ReplaceState(&self, cx: *mut JSContext, data: HandleValue, title: DOMString, url: Option<DOMString>) {
        let data = StructuredCloneData::write(cx, data).unwrap();
        self.window.browsing_context().replace_state(data, title, url);
    }
}
