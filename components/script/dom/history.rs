/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HistoryBinding::{self, HistoryMethods, ScrollRestoration};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use js::jsapi::{JSContext, HandleValue};
use js::jsval::JSVal;
use util::str::DOMString;

#[dom_struct]
pub struct History {
    reflector_: Reflector,
}

impl History {
    pub fn new_inherited() -> History {
        History {
            reflector_: Reflector::new(),
        }
    }

    pub fn new(global: GlobalRef) -> Root<History> {
        reflect_dom_object(box History::new_inherited(),
                           global,
                           HistoryBinding::Wrap)
    }
}

impl HistoryMethods for History {
    // https://html.spec.whatwg.org/multipage/#dom-history-length
    fn Length(&self) -> u32 {
        0
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-scroll-restoration
    fn ScrollRestoration(&self) -> ScrollRestoration {
        ScrollRestoration::Auto
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-scroll-restoration
    fn SetScrollRestoration(&self, value: ScrollRestoration) {

    }

    // https://html.spec.whatwg.org/multipage/#dom-history-state
    fn State(&self, cx: *mut JSContext) -> JSVal {
        unreachable!()
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-go
    fn Go(&self, delta: i32) {

    }

    // https://html.spec.whatwg.org/multipage/#dom-history-back
    fn Back(&self) {

    }

    // https://html.spec.whatwg.org/multipage/#dom-history-forward
    fn Forward(&self) {

    }

    // https://html.spec.whatwg.org/multipage/#dom-history-pushstate
    fn PushState(&self, cx: *mut JSContext, data: HandleValue, title: DOMString, url: Option<DOMString>) {

    }

    // https://html.spec.whatwg.org/multipage/#dom-history-replacestate
    fn ReplaceState(&self, cx: *mut JSContext, data: HandleValue, title: DOMString, url: Option<DOMString>) {

    }
}
