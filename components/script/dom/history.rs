/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HistoryBinding::{self, HistoryMethods, ScrollRestoration};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::structuredclone::StructuredCloneData;
use dom::window::Window;
use msg::constellation_msg::{ConstellationChan, NavigationDirection};
use script_traits::ScriptMsg as ConstellationMsg;
use ipc_channel::ipc;
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

    fn navigate(&self, direction: NavigationDirection) {
        let pipeline_info = self.window.parent_info();
        let ConstellationChan(ref chan) = *self.window.constellation_chan();
        let msg = ConstellationMsg::Navigate(pipeline_info, direction);
        chan.send(msg).unwrap();
    }
}

impl HistoryMethods for History {
    // https://html.spec.whatwg.org/multipage/#dom-history-length
    fn Length(&self) -> u32 {
        let (sender, receiver) = ipc::channel::<Option<usize>>().expect("Failed to create IPC channel");
        let pipeline_info = self.window.parent_info();
        let ConstellationChan(ref chan) = *self.window.constellation_chan();
        let msg = ConstellationMsg::HistoryLength(pipeline_info, sender);
        chan.send(msg).unwrap();

        match receiver.recv().unwrap() {
            Some(len) => len as u32,
            _ => 0
        }
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
        let direction = match delta {
            delta if delta > 0 => NavigationDirection::Forward(delta as u32),
            delta if delta < 0 => NavigationDirection::Back((-delta) as u32),
            _ => {
                // TODO: Reload page
                // This is assumed to be 0
                NavigationDirection::Forward(0)
            },
        };
        self.navigate(direction);
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-back
    fn Back(&self) {
        self.navigate(NavigationDirection::Back(1));
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-forward
    fn Forward(&self) {
        self.navigate(NavigationDirection::Forward(1));
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
