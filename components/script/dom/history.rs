/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::HistoryBinding;
use dom::bindings::codegen::Bindings::HistoryBinding::HistoryMethods;
use dom::bindings::codegen::Bindings::LocationBinding::LocationMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::error::{Error, ErrorResult};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::str::{DOMString, USVString};
use dom::bindings::structuredclone::StructuredCloneData;
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;
use dom::popstateevent::PopStateEvent;
use dom::window::Window;
use ipc_channel::ipc;
use js::jsapi::{HandleValue, JSAutoCompartment, JSContext, Heap};
use js::jsval::{JSVal, NullValue, UndefinedValue};
use msg::constellation_msg::{StateId, TraversalDirection};
use script_traits::{PushOrReplaceState, ScriptMsg as ConstellationMsg};
use servo_url::ServoUrl;
use std::cell::Cell;
use std::collections::HashMap;
use url::Position;

// https://html.spec.whatwg.org/multipage/#the-history-interface
#[dom_struct]
pub struct History {
    reflector_: Reflector,
    window: JS<Window>,
    history_entries: DOMRefCell<HashMap<StateId, HistoryEntry>>,
    active_state: Cell<Option<StateId>>,
}

impl History {
    pub fn new_inherited(window: &Window) -> History {
        History {
            reflector_: Reflector::new(),
            window: JS::from_ref(&window),
            history_entries: DOMRefCell::new(HashMap::new()),
            active_state: Cell::new(None),
        }
    }

    pub fn new(window: &Window) -> Root<History> {
        reflect_dom_object(box History::new_inherited(window),
                           window,
                           HistoryBinding::Wrap)
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-pushstate
    // https://html.spec.whatwg.org/multipage/#dom-history-replacestate
    fn update_state(&self,
                 cx: *mut JSContext,
                 data: HandleValue,
                 url: Option<USVString>,
                 replace: PushOrReplaceState) -> ErrorResult {
        // Step 1.
        let document = self.window.Document();

        // Step 2.
        if !document.is_fully_active() {
            return Err(Error::Security);
        }

        // TODO Step 3. Optionally abort these steps if push/replace state is being abused.

        // Step 4-5.
        let cloned_data = try!(StructuredCloneData::write(cx, data));

        let global_scope = self.window.upcast::<GlobalScope>();

        let url = match url {
            Some(url) => {
                // Step 6.
                let global_url = global_scope.get_url();
                // 6.1
                let url = match global_url.join(&url.0) {
                    // 6.2
                    Err(_) => return Err(Error::Security),
                    // 6.3
                    Ok(url) => url,
                };

                let document_url = document.url();

                // 6.4
                if url[Position::BeforeScheme..Position::AfterPort] !=
                    document_url[Position::BeforeScheme..Position::AfterPort] {
                    return Err(Error::Security);
                }

                // 6.5
                if url.origin() != document_url.origin() && (url[Position::BeforePath..Position::AfterQuery]
                    != document_url[Position::BeforePath..Position::AfterQuery]) {
                    return Err(Error::Security);
                }
                url
            },
            // Step 7.
            None => document.url(),
        };

        // Step 8.
        let new_entry = HistoryEntry::new(cloned_data);
        let mut history_entries = self.history_entries.borrow_mut();
        let state_id = match replace {
            PushOrReplaceState::Push => {
                let new_state_id = StateId::new();
                self.active_state.set(Some(new_state_id));
                history_entries.insert(new_state_id, new_entry);
                new_state_id
            },
            PushOrReplaceState::Replace => {
                let current_state_id = match self.active_state.get() {
                    Some(state_id) => state_id,
                    None => StateId::new(),
                };
                self.active_state.set(Some(current_state_id));
                history_entries.insert(current_state_id, new_entry);
                current_state_id
            }
        };
        // Notify Constellation
        let msg = ConstellationMsg::HistoryStateChanged(global_scope.pipeline_id(), state_id, url.clone(), replace);
        let _ = global_scope.constellation_chan().send(msg);

        // Step 10.
        // TODO(cbrewster): We can set the document url without ever notifying the constellation
        // This seems like it could be bad in the case of reloading a document that was discarded due to
        // being in the distant history.
        document.set_url(url);
        Ok(())
    }

    pub fn set_active_state(&self, state_id: Option<StateId>, url: ServoUrl) {
        if state_id == self.active_state.get() {
            return;
        }

        self.active_state.set(state_id);
        let handle = self.get_state().unwrap_or(Heap::new(NullValue()).handle());
        self.window.Document().set_url(url);
        PopStateEvent::dispatch_jsval(self.window.upcast::<EventTarget>(), &*self.window, handle);
    }

    pub fn remove_states(&self, state_ids: Vec<StateId>) {
        let mut history_entries = self.history_entries.borrow_mut();
        for state_id in state_ids {
            history_entries.remove(&state_id);
        }
    }

    fn get_state(&self) -> Option<HandleValue> {
        let state_id = match self.active_state.get() {
            Some(id) => id,
            None => return None,
        };
        let global_scope = self.window.upcast::<GlobalScope>();
        let mut history_entries = self.history_entries.borrow_mut();
        let mut entry = history_entries.get_mut(&state_id).expect("Could not get active state.");
        Some(entry.get_state(global_scope))
    }

    /// Take all of the history state entries and write cached entries back into StructuredCloneData
    pub fn suspend(&self) {
                let global_scope = self.window.upcast::<GlobalScope>();
        let mut history_entries = self.history_entries.borrow_mut();
        for (_, entry) in history_entries.iter_mut() {
            entry.write_structured_clone(global_scope);
        }
    }
}

impl History {
    fn traverse_history(&self, direction: TraversalDirection) {
        let global_scope = self.window.upcast::<GlobalScope>();
        let pipeline = global_scope.pipeline_id();
        let msg = ConstellationMsg::TraverseHistory(Some(pipeline), direction);
        let _ = global_scope.constellation_chan().send(msg);
    }
}

impl HistoryMethods for History {
    // https://html.spec.whatwg.org/multipage/#dom-history-length
    fn Length(&self) -> u32 {
        let global_scope = self.window.upcast::<GlobalScope>();
        let pipeline = global_scope.pipeline_id();
        let (sender, recv) = ipc::channel().expect("Failed to create channel to send jsh length.");
        let msg = ConstellationMsg::JointSessionHistoryLength(pipeline, sender);
        let _ = global_scope.constellation_chan().send(msg);
        recv.recv().unwrap()
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-history-state
    unsafe fn State(&self, _cx: *mut JSContext) -> JSVal {
        self.get_state().map(|handle| handle.get()).unwrap_or(NullValue())
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-go
    fn Go(&self, delta: i32) {
        let direction = if delta > 0 {
            TraversalDirection::Forward(delta as usize)
        } else if delta < 0 {
            TraversalDirection::Back(-delta as usize)
        } else {
            self.window.Location().Reload();
            return;
        };

        self.traverse_history(direction);
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-back
    fn Back(&self) {
        self.traverse_history(TraversalDirection::Back(1));
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-forward
    fn Forward(&self) {
        self.traverse_history(TraversalDirection::Forward(1));
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-history-pushstate
    unsafe fn PushState(&self,
                        cx: *mut JSContext,
                        data: HandleValue,
                        _title: DOMString,
                        url: Option<USVString>) -> ErrorResult {
        self.update_state(cx, data, url, PushOrReplaceState::Push)
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-history-replacestate
    unsafe fn ReplaceState(&self,
                           cx: *mut JSContext,
                           data: HandleValue,
                           _title: DOMString,
                           url: Option<USVString>) -> ErrorResult {
        self.update_state(cx, data, url, PushOrReplaceState::Replace)
    }
}

/// Stores the JS state value for a given history entry. Any time the History's window is suspended
/// the cached data must be serialized into StructuredCloneData again.
#[derive(HeapSizeOf, JSTraceable)]
enum StateCache {
    /// The state is a structured clone, next time it is accessed, it will be deserialized and
    /// cached in the Cached variant. It is stored as an optional as reading the StructuredCloneData
    /// requires taking ownership of it.
    StructuredClone(Option<StructuredCloneData>),
    /// The cached state after deserialization
    Cached(Heap<JSVal>),
}

#[derive(HeapSizeOf, JSTraceable)]
struct HistoryEntry {
    state: StateCache,
}

impl HistoryEntry {
    fn new(cloned_data: StructuredCloneData) -> HistoryEntry {
        HistoryEntry {
            state: StateCache::StructuredClone(Some(cloned_data)),
        }
    }

    fn get_state(&mut self, global_scope: &GlobalScope) -> HandleValue {
        let state = match self.state {
            StateCache::Cached(ref state) => return state.handle(),
            StateCache::StructuredClone(ref mut cloned_data) => {
                let cloned_data = cloned_data.take().expect("StructuredClone cannot be None");
                let cx = global_scope.get_cx();
                rooted!(in(cx) let mut state = UndefinedValue());
                let globalhandle = global_scope.reflector().get_jsobject();
                let _ac = JSAutoCompartment::new(cx, globalhandle.get());
                cloned_data.read(global_scope, state.handle_mut());
                Heap::new(state.get())
            }
        };
        self.state = StateCache::Cached(state);
        if let StateCache::Cached(ref state) = self.state {
            return state.handle();
        }
        unreachable!()
    }

    fn write_structured_clone(&mut self, global_scope: &GlobalScope) {
        let clonded_data = match self.state {
            StateCache::StructuredClone(..) => return,
            StateCache::Cached(ref state) => {
                StructuredCloneData::write(global_scope.get_cx(), state.handle())
            }
        };
        match clonded_data {
            Ok(clonded_data) => {
                self.state = StateCache::StructuredClone(Some(clonded_data))
            },
            Err(e) => warn!("Failed to write structured clone {:?}", e),
        }
    }
}
