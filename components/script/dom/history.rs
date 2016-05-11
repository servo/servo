/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::HistoryBinding::{self, HistoryMethods, ScrollRestoration};
use dom::bindings::codegen::Bindings::LocationBinding::LocationMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::error::{Error, ErrorResult, Fallible};
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

    fn traverse_history(&self, direction: NavigationDirection) {
        let ConstellationChan(ref chan) = *self.window.constellation_chan();
        let msg = ConstellationMsg::Navigate(direction);
        chan.send(msg).unwrap();
    }

    fn is_fully_active(&self) -> ErrorResult {
        let (sender, receiver) = ipc::channel::<bool>().expect("Failed to create IPC channel");
        let ConstellationChan(ref chan) = *self.window.constellation_chan();
        let msg = ConstellationMsg::IsPipelineFullyActive(self.window.pipeline(), sender);
        chan.send(msg).unwrap();

        if receiver.recv().unwrap() {
            Ok(())
        } else {
            Err(Error::Security)
        }
    }
}

impl HistoryMethods for History {
    // https://html.spec.whatwg.org/multipage/#dom-history-length
    fn GetLength(&self) -> Fallible<u32> {
        try!(self.is_fully_active());
        let (sender, receiver) = ipc::channel::<usize>().expect("Failed to create IPC channel");
        let ConstellationChan(ref chan) = *self.window.constellation_chan();
        let msg = ConstellationMsg::HistoryLength(sender);
        chan.send(msg).unwrap();
        Ok(receiver.recv().unwrap() as u32)
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-scroll-restoration
    fn GetScrollRestoration(&self) -> Fallible<ScrollRestoration> {
        try!(self.is_fully_active());
        Ok(ScrollRestoration::Auto)
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-scroll-restoration
    fn SetScrollRestoration(&self, _value: ScrollRestoration) -> ErrorResult {
        try!(self.is_fully_active());
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-state
    fn GetState(&self, cx: *mut JSContext) -> Fallible<JSVal> {
        try!(self.is_fully_active());
        let state = self.window.browsing_context().state();
        let _ac = JSAutoCompartment::new(cx, self.reflector_.get_jsobject().get());
        let mut state_js = RootedValue::new(cx, UndefinedValue());
        state.read(GlobalRef::Window(&self.window), state_js.handle_mut());
        Ok(state_js.handle().get())
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-go
    fn Go(&self, delta: i32) -> ErrorResult {
        try!(self.is_fully_active());
        let direction = match delta {
            delta if delta > 0 => NavigationDirection::Forward(delta as usize),
            delta if delta < 0 => NavigationDirection::Back((-delta) as usize),
            _ => {
                self.window.Document().GetLocation().map(|location| location.Reload());
                return Ok(());
            },
        };
        self.traverse_history(direction);
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-back
    fn Back(&self) -> ErrorResult {
        try!(self.is_fully_active());
        self.traverse_history(NavigationDirection::Back(1));
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-forward
    fn Forward(&self) -> ErrorResult {
        try!(self.is_fully_active());
        self.traverse_history(NavigationDirection::Forward(1));
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-pushstate
    fn PushState(&self,
                 cx: *mut JSContext,
                 data: HandleValue,
                 title: DOMString,
                 url: Option<DOMString>)
                 -> ErrorResult {
        try!(self.is_fully_active());
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
        try!(self.is_fully_active());
        let data = try!(StructuredCloneData::write(cx, data));
        try!(self.window.browsing_context().replace_state(data, title, url));
        Ok(())
    }
}
