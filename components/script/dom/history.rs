/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HistoryBinding::{self, HistoryMethods, ScrollRestoration};
use dom::bindings::error::ErrorResult;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::structuredclone::StructuredCloneData;
use dom::window::Window;
use ipc_channel::ipc;
use js::jsapi::{HandleValue, JSContext, JSAutoCompartment, RootedValue};
use js::jsval::{JSVal, UndefinedValue};
use msg::constellation_msg::{ConstellationChan, NavigationDirection};
use script_traits::ScriptMsg as ConstellationMsg;
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
        let ConstellationChan(ref chan) = *self.window.constellation_chan();
        let msg = ConstellationMsg::Navigate(direction);
        chan.send(msg).unwrap();
    }
}

impl HistoryMethods for History {
    // https://html.spec.whatwg.org/multipage/#dom-history-length
    fn Length(&self) -> u32 {
        // TODO: Check if `Document` is `fully active`
        let (sender, receiver) = ipc::channel::<usize>().expect("Failed to create IPC channel");
        let ConstellationChan(ref chan) = *self.window.constellation_chan();
        let msg = ConstellationMsg::HistoryLength(sender);
        chan.send(msg).unwrap();

        receiver.recv().unwrap() as u32
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-scroll-restoration
    fn ScrollRestoration(&self) -> ScrollRestoration {
        // TODO: Check if `Document` is `fully active`
        ScrollRestoration::Auto
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-scroll-restoration
    fn SetScrollRestoration(&self, _value: ScrollRestoration) {
        // TODO: Check if `Document` is `fully active`

    }

    // https://html.spec.whatwg.org/multipage/#dom-history-state
    fn State(&self, cx: *mut JSContext) -> JSVal {
        // TODO: Check if `Document` is `fully active`
        let state = self.window.browsing_context().state();
        let _ac = JSAutoCompartment::new(cx, self.reflector_.get_jsobject().get());
        let mut state_js = RootedValue::new(cx, UndefinedValue());
        state.read(GlobalRef::Window(&self.window), state_js.handle_mut());
        state_js.handle().get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-go
    fn Go(&self, delta: i32) {
        // TODO: Check if `Document` is `fully active`
        let direction = match delta {
            delta if delta > 0 => NavigationDirection::Forward(delta as usize),
            delta if delta < 0 => NavigationDirection::Back((-delta) as usize),
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
        // TODO: Check if `Document` is `fully active`
        self.navigate(NavigationDirection::Back(1));
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-forward
    fn Forward(&self) {
        // TODO: Check if `Document` is `fully active`
        self.navigate(NavigationDirection::Forward(1));
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-pushstate
    fn PushState(&self,
                 cx: *mut JSContext,
                 data: HandleValue,
                 title: DOMString,
                 url: Option<DOMString>)
                 -> ErrorResult {
        // TODO: Check if `Document` is `fully active`
        let data = try!(StructuredCloneData::write(cx, data));
        try!(self.window.browsing_context().push_state(data, title, url));
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-replacestate
    fn ReplaceState(&self,
                    cx: *mut JSContext,
                    data: HandleValue,
                    title: DOMString,
                    url: Option<DOMString>)
                    -> ErrorResult {
        // TODO: Check if `Document` is `fully active`
        let data = try!(StructuredCloneData::write(cx, data));
        try!(self.window.browsing_context().replace_state(data, title, url));
        Ok(())
    }
}
