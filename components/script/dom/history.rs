/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::cmp::Ordering;

use base::id::HistoryStateId;
use dom_struct::dom_struct;
use embedder_traits::TraversalDirection;
use js::jsapi::Heap;
use js::jsval::{JSVal, NullValue, UndefinedValue};
use js::rust::{HandleValue, MutableHandleValue};
use net_traits::{CoreResourceMsg, IpcSend};
use profile_traits::ipc;
use profile_traits::ipc::channel;
use script_traits::{ScriptMsg, StructuredSerializedData};
use servo_url::ServoUrl;

use crate::dom::bindings::codegen::Bindings::HistoryBinding::HistoryMethods;
use crate::dom::bindings::codegen::Bindings::LocationBinding::Location_Binding::LocationMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::structuredclone;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::hashchangeevent::HashChangeEvent;
use crate::dom::popstateevent::PopStateEvent;
use crate::dom::window::Window;
use crate::script_runtime::{CanGc, JSContext};

enum PushOrReplace {
    Push,
    Replace,
}

/// <https://html.spec.whatwg.org/multipage/#the-history-interface>
#[dom_struct]
pub(crate) struct History {
    reflector_: Reflector,
    window: Dom<Window>,
    #[ignore_malloc_size_of = "mozjs"]
    state: Heap<JSVal>,
    #[no_trace]
    state_id: Cell<Option<HistoryStateId>>,
}

impl History {
    pub(crate) fn new_inherited(window: &Window) -> History {
        let state = Heap::default();
        state.set(NullValue());
        History {
            reflector_: Reflector::new(),
            window: Dom::from_ref(window),
            state,
            state_id: Cell::new(None),
        }
    }

    pub(crate) fn new(window: &Window) -> DomRoot<History> {
        reflect_dom_object(
            Box::new(History::new_inherited(window)),
            window,
            CanGc::note(),
        )
    }
}

impl History {
    fn traverse_history(&self, direction: TraversalDirection) -> ErrorResult {
        if !self.window.Document().is_fully_active() {
            return Err(Error::Security);
        }
        let msg = ScriptMsg::TraverseHistory(direction);
        let _ = self
            .window
            .as_global_scope()
            .script_to_constellation_chan()
            .send(msg);
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#history-traversal>
    /// Steps 5-16
    #[allow(unsafe_code)]
    pub(crate) fn activate_state(
        &self,
        state_id: Option<HistoryStateId>,
        url: ServoUrl,
        can_gc: CanGc,
    ) {
        // Steps 5
        let document = self.window.Document();
        let old_url = document.url().clone();
        document.set_url(url.clone());

        // Step 6
        let hash_changed = old_url.fragment() != url.fragment();

        // Step 8
        if let Some(fragment) = url.fragment() {
            document.check_and_scroll_fragment(fragment, can_gc);
        }

        // Step 11
        let state_changed = state_id != self.state_id.get();
        self.state_id.set(state_id);
        let serialized_data = match state_id {
            Some(state_id) => {
                let (tx, rx) = ipc::channel(self.global().time_profiler_chan().clone()).unwrap();
                let _ = self
                    .window
                    .as_global_scope()
                    .resource_threads()
                    .send(CoreResourceMsg::GetHistoryState(state_id, tx));
                rx.recv().unwrap()
            },
            None => None,
        };

        match serialized_data {
            Some(data) => {
                let data = StructuredSerializedData {
                    serialized: data,
                    ports: None,
                    blobs: None,
                };
                rooted!(in(*GlobalScope::get_cx()) let mut state = UndefinedValue());
                if structuredclone::read(self.window.as_global_scope(), data, state.handle_mut())
                    .is_err()
                {
                    warn!("Error reading structuredclone data");
                }
                self.state.set(state.get());
            },
            None => {
                self.state.set(NullValue());
            },
        }

        // TODO: Queue events on DOM Manipulation task source if non-blocking flag is set.
        // Step 16.1
        if state_changed {
            PopStateEvent::dispatch_jsval(
                self.window.upcast::<EventTarget>(),
                &self.window,
                unsafe { HandleValue::from_raw(self.state.handle()) },
                can_gc,
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
                url.into_string(),
                can_gc,
            );
            event
                .upcast::<Event>()
                .fire(self.window.upcast::<EventTarget>(), can_gc);
        }
    }

    pub(crate) fn remove_states(&self, states: Vec<HistoryStateId>) {
        let _ = self
            .window
            .as_global_scope()
            .resource_threads()
            .send(CoreResourceMsg::RemoveHistoryStates(states));
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-history-pushstate>
    /// <https://html.spec.whatwg.org/multipage/#dom-history-replacestate>
    fn push_or_replace_state(
        &self,
        cx: JSContext,
        data: HandleValue,
        _title: DOMString,
        url: Option<USVString>,
        push_or_replace: PushOrReplace,
    ) -> ErrorResult {
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
        let serialized_data = structuredclone::write(cx, data, None)?;

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
            None => document.url(),
        };

        // Step 8
        let state_id = match push_or_replace {
            PushOrReplace::Push => {
                let state_id = HistoryStateId::new();
                self.state_id.set(Some(state_id));
                let msg = ScriptMsg::PushHistoryState(state_id, new_url.clone());
                let _ = self
                    .window
                    .as_global_scope()
                    .script_to_constellation_chan()
                    .send(msg);
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
                let _ = self
                    .window
                    .as_global_scope()
                    .script_to_constellation_chan()
                    .send(msg);
                state_id
            },
        };

        let _ = self.window.as_global_scope().resource_threads().send(
            CoreResourceMsg::SetHistoryState(state_id, serialized_data.serialized.clone()),
        );

        // TODO: Step 9 Update current entry to represent a GET request
        // https://github.com/servo/servo/issues/19156

        // Step 10
        document.set_url(new_url);

        // Step 11
        rooted!(in(*cx) let mut state = UndefinedValue());
        if structuredclone::read(
            self.window.as_global_scope(),
            serialized_data,
            state.handle_mut(),
        )
        .is_err()
        {
            warn!("Error reading structuredclone data");
        }

        // Step 12
        self.state.set(state.get());

        // TODO: Step 13 Update Document's latest entry to current entry
        // https://github.com/servo/servo/issues/19158

        Ok(())
    }
}

impl HistoryMethods<crate::DomTypeHolder> for History {
    /// <https://html.spec.whatwg.org/multipage/#dom-history-state>
    fn GetState(&self, _cx: JSContext, mut retval: MutableHandleValue) -> Fallible<()> {
        if !self.window.Document().is_fully_active() {
            return Err(Error::Security);
        }
        retval.set(self.state.get());
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-history-length>
    fn GetLength(&self) -> Fallible<u32> {
        if !self.window.Document().is_fully_active() {
            return Err(Error::Security);
        }
        let (sender, recv) = channel(self.global().time_profiler_chan().clone())
            .expect("Failed to create channel to send jsh length.");
        let msg = ScriptMsg::JointSessionHistoryLength(sender);
        let _ = self
            .window
            .as_global_scope()
            .script_to_constellation_chan()
            .send(msg);
        Ok(recv.recv().unwrap())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-history-go>
    fn Go(&self, delta: i32, can_gc: CanGc) -> ErrorResult {
        let direction = match delta.cmp(&0) {
            Ordering::Greater => TraversalDirection::Forward(delta as usize),
            Ordering::Less => TraversalDirection::Back(-delta as usize),
            Ordering::Equal => return self.window.Location().Reload(can_gc),
        };

        self.traverse_history(direction)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-history-back>
    fn Back(&self) -> ErrorResult {
        self.traverse_history(TraversalDirection::Back(1))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-history-forward>
    fn Forward(&self) -> ErrorResult {
        self.traverse_history(TraversalDirection::Forward(1))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-history-pushstate>
    fn PushState(
        &self,
        cx: JSContext,
        data: HandleValue,
        title: DOMString,
        url: Option<USVString>,
    ) -> ErrorResult {
        self.push_or_replace_state(cx, data, title, url, PushOrReplace::Push)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-history-replacestate>
    fn ReplaceState(
        &self,
        cx: JSContext,
        data: HandleValue,
        title: DOMString,
        url: Option<USVString>,
    ) -> ErrorResult {
        self.push_or_replace_state(cx, data, title, url, PushOrReplace::Replace)
    }
}
