/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HistoryBinding;
use dom::bindings::codegen::Bindings::HistoryBinding::HistoryMethods;
use dom::bindings::codegen::Bindings::LocationBinding::LocationMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::error::{Error, ErrorResult};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::{DOMString, USVString};
use dom::bindings::structuredclone::StructuredCloneData;
use dom::globalscope::GlobalScope;
use dom::window::Window;
use ipc_channel::ipc;
use js::jsapi::{HandleValue, JSContext, Heap};
use js::jsval::{JSVal, NullValue};
use msg::constellation_msg::{StateId, TraversalDirection};
use script_traits::ScriptMsg as ConstellationMsg;
use servo_url::ServoUrl;
use std::cell::Cell;
use std::collections::HashMap;
use url::Position;

enum PushOrReplace {
    Push,
    Replace,
}

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
                 title: DOMString,
                 url: Option<USVString>,
                 replace: PushOrReplace) -> ErrorResult {
        // Step 1.
        let document = self.window.Document();

        // Step 2.
        if !document.is_fully_active() {
            return Err(Error::Security);
        }

        // TODO Step 3. Optionally abort these steps if push/replace state is being abused.

        // Step 4-5.
        let cloned_data = try!(StructuredCloneData::write(cx, data));
        rooted!(in(cx) let mut state = NullValue());
        cloned_data.read(self.window.upcast::<GlobalScope>(), state.handle_mut());

        let url = match url {
            Some(url) => {
                // Step 6.
                let global_url = self.window.upcast::<GlobalScope>().get_url();
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
        let new_entry = HistoryEntry::new(state.get(), url.clone(), title);
        let mut history_entries = self.history_entries.borrow_mut();
        match replace {
            PushOrReplace::Push => {
                let new_state_id = StateId::new();
                self.active_state.set(Some(new_state_id));
                history_entries.insert(new_state_id, new_entry);
                // Notify Constellation
            },
            PushOrReplace::Replace => {
                let current_state_id = match self.active_state.get() {
                    Some(state_id) => state_id,
                    None => StateId::new(),
                };
                history_entries.insert(current_state_id, new_entry);
                // Notify Constellation
            }
        }

        // Step 10.
        document.set_url(url);
        Ok(())
    }

    pub fn set_active_state(&self, state_id: Option<StateId>) {
        self.active_state.set(state_id);
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
        let history_entries = self.history_entries.borrow();
        match self.active_state.get().and_then(|state_id| history_entries.get(&state_id)) {
            Some(entry) => {
                entry.state.get()
            },
            None => NullValue(),
        }
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
                        title: DOMString,
                        url: Option<USVString>) -> ErrorResult {
        self.update_state(cx, data, title, url, PushOrReplace::Push)
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-history-replacestate
    unsafe fn ReplaceState(&self,
                           cx: *mut JSContext,
                           data: HandleValue,
                           title: DOMString,
                           url: Option<USVString>) -> ErrorResult {
        self.update_state(cx, data, title, url, PushOrReplace::Replace)
    }
}

#[derive(HeapSizeOf, JSTraceable)]
struct HistoryEntry {
    url: ServoUrl,
    title: DOMString,
    state: Heap<JSVal>,
}

impl HistoryEntry {
    fn new(state: JSVal, url: ServoUrl, title: DOMString) -> HistoryEntry {
        HistoryEntry {
            url: url,
            title: title,
            state: Heap::new(state),
        }
    }
}