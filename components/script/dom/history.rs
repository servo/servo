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
use dom::event::Event;
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;
use dom::hashchangeevent::HashChangeEvent;
use dom::popstateevent::PopStateEvent;
use dom::window::Window;
use dom_struct::dom_struct;
use js::jsapi::{Heap, JSContext};
use js::jsval::{JSVal, NullValue, UndefinedValue};
use js::rust::HandleValue;
use msg::constellation_msg::{HistoryStateId, TraversalDirection};
use net_traits::{CoreResourceMsg, IpcSend};
use profile_traits::ipc;
use profile_traits::ipc::channel;
use script_traits::ScriptMsg;
use servo_url::ServoUrl;
use std::cell::Cell;
use typeholder::TypeHolderTrait;

enum PushOrReplace {
    Push,
    Replace,
}

// https://html.spec.whatwg.org/multipage/#the-history-interface
#[dom_struct]
pub struct History<TH: TypeHolderTrait> {
    reflector_: Reflector<TH>,
    window: Dom<Window<TH>>,
    state: Heap<JSVal>,
    state_id: Cell<Option<HistoryStateId>>,
}

impl <TH: TypeHolderTrait> History<TH> {
    pub fn new_inherited(window: &Window<TH>) -> History<TH> {
        let state = Heap::default();
        state.set(NullValue());
        History {
            reflector_: Reflector::new(),
            window: Dom::from_ref(&window),
            state: state,
            state_id: Cell::new(None),
        }
    }

    pub fn new(window: &Window<TH>) -> DomRoot<History<TH>> {
        reflect_dom_object(Box::new(History::new_inherited(window)),
                           window,
                           HistoryBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> History<TH> {
    fn traverse_history(&self, direction: TraversalDirection) -> ErrorResult {
        if !self.window.Document().is_fully_active() {
            return Err(Error::Security);
        }
        let msg = ScriptMsg::TraverseHistory(direction);
        let _ = self.window.upcast::<GlobalScope<TH>>().script_to_constellation_chan().send(msg);
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#history-traversal
    // Steps 5-16
    #[allow(unsafe_code)]
    pub fn activate_state(&self, state_id: Option<HistoryStateId>, url: ServoUrl) {
        // Steps 5
        let document = self.window.Document();
        let old_url = document.url().clone();
        document.set_url(url.clone());

        // Step 6
        let hash_changed =  old_url.fragment() != url.fragment();

        // Step 8
        if let Some(fragment) = url.fragment() {
            document.check_and_scroll_fragment(fragment);
        }

        // Step 11
        let state_changed = state_id != self.state_id.get();
        self.state_id.set(state_id);
        let serialized_data = match state_id {
            Some(state_id) => {
                let (tx, rx) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();
                let _ = self.window
                    .upcast::<GlobalScope<TH>>()
                    .resource_threads()
                    .send(CoreResourceMsg::GetHistoryState(state_id, tx));
                rx.recv().unwrap()
            },
            None => None,
        };

        match serialized_data {
            Some(serialized_data) => {
                let global_scope = self.window.upcast::<GlobalScope<TH>>();
                rooted!(in(global_scope.get_cx()) let mut state = UndefinedValue());
                StructuredCloneData::Vector(serialized_data).read(&global_scope, state.handle_mut());
                self.state.set(state.get());
            },
            None => {
                self.state.set(NullValue());
            }
        }

        // TODO: Queue events on DOM Manipulation task source if non-blocking flag is set.
        // Step 16.1
        if state_changed {
            PopStateEvent::dispatch_jsval(
                self.window.upcast::<EventTarget<TH>>(),
                &*self.window,
                unsafe { HandleValue::from_raw(self.state.handle()) }
            );
        }

        // Step 16.3
        if hash_changed {
            let event = HashChangeEvent::new(
                &self.window,
                atom!("hashchange"),
                false,
                false,
                old_url.into_string(),
                url.into_string());
            event.upcast::<Event<TH>>().fire(self.window.upcast::<EventTarget<TH>>());
        }
    }

    pub fn remove_states(&self, states: Vec<HistoryStateId>) {
        let _ = self.window
            .upcast::<GlobalScope<TH>>()
            .resource_threads()
            .send(CoreResourceMsg::RemoveHistoryStates(states));
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-pushstate
    // https://html.spec.whatwg.org/multipage/#dom-history-replacestate
    fn push_or_replace_state(&self,
                             cx: *mut JSContext,
                             data: HandleValue,
                             _title: DOMString,
                             url: Option<USVString>,
                             push_or_replace: PushOrReplace) -> ErrorResult {
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
        let serialized_data = StructuredCloneData::<TH>::write(cx, data)?.move_to_arraybuffer();

        let new_url: ServoUrl = match url {
            // Step 6
            Some(urlstring) => {
                let document_url = document.url();

                // Step 6.1
                let new_url = match ServoUrl::parse_with_base(Some(&document_url), &urlstring.0) {
                    // Step 6.3
                    Ok(parsed_url) => parsed_url,
                    // Step 6.2
                    Err(_) => return Err(Error::Security),
                };

                // Step 6.4
                if new_url.scheme() != document_url.scheme() ||
                   new_url.host() != document_url.host() ||
                   new_url.port() != document_url.port() ||
                   new_url.username() != document_url.username() ||
                   new_url.password() != document_url.password()
                {
                    return Err(Error::Security);
                }

                // Step 6.5
                if new_url.origin() != document_url.origin() {
                    return Err(Error::Security);
                }

                new_url
            },
            // Step 7
            None => {
                document.url()
            }
        };

        // Step 8
        let state_id = match push_or_replace {
            PushOrReplace::Push => {
                let state_id = HistoryStateId::new();
                self.state_id.set(Some(state_id));
                let msg = ScriptMsg::PushHistoryState(state_id, new_url.clone());
                let _ = self.window.upcast::<GlobalScope<TH>>().script_to_constellation_chan().send(msg);
                state_id
            },
            PushOrReplace::Replace => {
                let state_id = match self.state_id.get() {
                    Some(state_id) => state_id,
                    None => {
                        let state_id = HistoryStateId::new();
                        self.state_id.set(Some(state_id));
                        state_id
                    },
                };
                let msg = ScriptMsg::ReplaceHistoryState(state_id, new_url.clone());
                let _ = self.window.upcast::<GlobalScope<TH>>().script_to_constellation_chan().send(msg);
                state_id
            },
        };

        let _ = self.window
            .upcast::<GlobalScope<TH>>()
            .resource_threads()
            .send(CoreResourceMsg::SetHistoryState(state_id, serialized_data.clone()));


        // TODO: Step 9 Update current entry to represent a GET request
        // https://github.com/servo/servo/issues/19156

        // Step 10
        document.set_url(new_url);

        // Step 11
        let global_scope = self.window.upcast::<GlobalScope<TH>>();
        rooted!(in(cx) let mut state = UndefinedValue());
        StructuredCloneData::Vector(serialized_data).read(&global_scope, state.handle_mut());

        // Step 12
        self.state.set(state.get());

        // TODO: Step 13 Update Document's latest entry to current entry
        // https://github.com/servo/servo/issues/19158

        Ok(())
    }
}

impl<TH: TypeHolderTrait> HistoryMethods for History<TH> {
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
        let _ = self.window.upcast::<GlobalScope<TH>>().script_to_constellation_chan().send(msg);
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
