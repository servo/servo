/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HistoryBinding;
use dom::bindings::codegen::Bindings::HistoryBinding::HistoryMethods;
use dom::bindings::codegen::Bindings::LocationBinding::LocationBinding::LocationMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot};
use dom::bindings::str::{DOMString, USVString};
use dom::bindings::structuredclone::StructuredCloneData;
use dom::globalscope::GlobalScope;
use dom::window::Window;
use dom_struct::dom_struct;
use js::jsapi::{Heap, JSContext};
use js::jsval::{JSVal, NullValue, UndefinedValue};
use js::rust::HandleValue;
use msg::constellation_msg::TraversalDirection;
use profile_traits::ipc::channel;
use script_traits::ScriptMsg;

enum PushOrReplace {
    Push,
    Replace,
}

// https://html.spec.whatwg.org/multipage/#the-history-interface
#[dom_struct]
pub struct History {
    reflector_: Reflector,
    window: Dom<Window>,
    state: Heap<JSVal>,
}

impl History {
    pub fn new_inherited(window: &Window) -> History {
        let state = Heap::default();
        state.set(NullValue());
        History {
            reflector_: Reflector::new(),
            window: Dom::from_ref(&window),
            state: state,
        }
    }

    pub fn new(window: &Window) -> DomRoot<History> {
        reflect_dom_object(Box::new(History::new_inherited(window)),
                           window,
                           HistoryBinding::Wrap)
    }
}

impl History {
    fn traverse_history(&self, direction: TraversalDirection) -> ErrorResult {
        if !self.window.Document().is_fully_active() {
            return Err(Error::Security);
        }
        let msg = ScriptMsg::TraverseHistory(direction);
        let _ = self.window.upcast::<GlobalScope>().script_to_constellation_chan().send(msg);
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-pushstate
    // https://html.spec.whatwg.org/multipage/#dom-history-replacestate
    fn push_or_replace_state(&self,
                             cx: *mut JSContext,
                             data: HandleValue,
                             _title: DOMString,
                             _url: Option<USVString>,
                             _push_or_replace: PushOrReplace) -> ErrorResult {
        // Step 1
        let document = self.window.Document();

        // Step 2
        if !document.is_fully_active() {
            return Err(Error::Security);
        }

        // TODO: Step 3 Optionally abort these steps
        // https://github.com/servo/servo/issues/19159

        // TODO: Step 4

        // Step 5
        let serialized_data = StructuredCloneData::write(cx, data)?;

        // TODO: Steps 6-7 Url Handling
        // https://github.com/servo/servo/issues/19157

        // TODO: Step 8 Push/Replace session history entry
        // https://github.com/servo/servo/issues/19156

        // TODO: Step 9 Update current entry to represent a GET request
        // https://github.com/servo/servo/issues/19156

        // TODO: Step 10 Set document's URL to new URL
        // https://github.com/servo/servo/issues/19157

        // Step 11
        let global_scope = self.window.upcast::<GlobalScope>();
        rooted!(in(cx) let mut state = UndefinedValue());
        serialized_data.read(&global_scope, state.handle_mut());

        // Step 12
        self.state.set(state.get());

        // TODO: Step 13 Update Document's latest entry to current entry
        // https://github.com/servo/servo/issues/19158

        Ok(())
    }
}

impl HistoryMethods for History {
    // https://html.spec.whatwg.org/multipage/#dom-history-state
    #[allow(unsafe_code)]
    unsafe fn GetState(&self, _cx: *mut JSContext) -> Fallible<JSVal> {
        if !self.window.Document().is_fully_active() {
            return Err(Error::Security);
        }
        Ok(self.state.get())
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-length
    fn GetLength(&self) -> Fallible<u32> {
        if !self.window.Document().is_fully_active() {
            return Err(Error::Security);
        }
        let (sender, recv) =
            channel(self.global().time_profiler_chan().clone()).expect("Failed to create channel to send jsh length.");
        let msg = ScriptMsg::JointSessionHistoryLength(sender);
        let _ = self.window.upcast::<GlobalScope>().script_to_constellation_chan().send(msg);
        Ok(recv.recv().unwrap())
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-go
    fn Go(&self, delta: i32) -> ErrorResult {
        let direction = if delta > 0 {
            TraversalDirection::Forward(delta as usize)
        } else if delta < 0 {
            TraversalDirection::Back(-delta as usize)
        } else {
            return self.window.Location().Reload();
        };

        self.traverse_history(direction)
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-back
    fn Back(&self) -> ErrorResult {
        self.traverse_history(TraversalDirection::Back(1))
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-forward
    fn Forward(&self) -> ErrorResult {
        self.traverse_history(TraversalDirection::Forward(1))
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-pushstate
    #[allow(unsafe_code)]
    unsafe fn PushState(&self,
                        cx: *mut JSContext,
                        data: HandleValue,
                        title: DOMString,
                        url: Option<USVString>) -> ErrorResult {
        self.push_or_replace_state(cx, data, title, url, PushOrReplace::Push)
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-replacestate
    #[allow(unsafe_code)]
    unsafe fn ReplaceState(&self,
                           cx: *mut JSContext,
                           data: HandleValue,
                           title: DOMString,
                           url: Option<USVString>) -> ErrorResult {
        self.push_or_replace_state(cx, data, title, url, PushOrReplace::Replace)
    }
}
