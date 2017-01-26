/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

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
use dom::hashchangeevent::HashChangeEvent;
use dom::popstateevent::PopStateEvent;
use dom::window::Window;
use ipc_channel::ipc;
use js::jsapi::{HandleValue, JSAutoCompartment, JSContext, Heap};
use js::jsval::{JSVal, NullValue};
use msg::constellation_msg::{StateId, TraversalDirection};
use net_traits::{CoreResourceMsg, IpcSend};
use script_traits::{PushOrReplaceState, ScriptMsg as ConstellationMsg};
use servo_url::ServoUrl;
use std::cell::Cell;
use url::Position;

// https://html.spec.whatwg.org/multipage/#the-history-interface
#[dom_struct]
pub struct History {
    reflector_: Reflector,
    window: JS<Window>,
    active_state: Heap<JSVal>,
    latest_state: Cell<Option<StateId>>,
}

impl History {
    pub fn new_inherited(window: &Window) -> History {
        History {
            reflector_: Reflector::new(),
            window: JS::from_ref(&window),
            active_state: Heap::new(NullValue()),
            latest_state: Cell::new(None),
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
        let state_id = match replace {
            PushOrReplaceState::Push => StateId::new(),
            PushOrReplaceState::Replace => self.latest_state.get().unwrap_or(StateId::new()),
        };
        self.latest_state.set(Some(state_id));

        let state_data = cloned_data.move_to_arraybuffer();
        let _ = global_scope.resource_threads().send(CoreResourceMsg::SetHistoryState(state_id, state_data.clone()));

        let cloned_data = StructuredCloneData::Vector(state_data);
        let cx = global_scope.get_cx();
        let globalhandle = global_scope.reflector().get_jsobject();
        rooted!(in(cx) let mut state = NullValue());
        let _ac = JSAutoCompartment::new(cx, globalhandle.get());
        cloned_data.read(global_scope, state.handle_mut());
        self.active_state.set(state.get());

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
        self.latest_state.set(state_id);
        match state_id {
            Some(state_id) => {
                let global_scope = self.window.upcast::<GlobalScope>();
                let (sender, receiver) = ipc::channel().expect("Failed to create IPC channel.");
                let _ = global_scope.resource_threads().send(CoreResourceMsg::GetHistoryState(state_id, sender));
                // The receiver could error out or the received state may be none.
                let state_data = receiver.recv().ok().and_then(|s| s).expect("Failed to get state data");
                let cloned_data = StructuredCloneData::Vector(state_data);

                let cx = global_scope.get_cx();
                let globalhandle = global_scope.reflector().get_jsobject();
                rooted!(in(cx) let mut state = NullValue());
                let _ac = JSAutoCompartment::new(cx, globalhandle.get());
                cloned_data.read(global_scope, state.handle_mut());
                self.active_state.set(state.get());
            }
            None => self.active_state.set(NullValue())
        };
        PopStateEvent::dispatch_jsval(self.window.upcast::<EventTarget>(), &*self.window, self.active_state.handle());

        let doc_url = self.window.Document().url();
        if doc_url.fragment() != url.fragment() {
            let old_url = doc_url.into_string();
            let new_url = url.clone().into_string();
            self.window.Document().set_url(url);
            HashChangeEvent::dispatch(self.window.upcast::<EventTarget>(), &*self.window, old_url, new_url);
        }
    }

    pub fn remove_states(&self, state_ids: Vec<StateId>) {
        let global_scope = self.window.upcast::<GlobalScope>();
        let _ = global_scope.resource_threads().send(CoreResourceMsg::RemoveHistoryStates(state_ids));
    }

    /// Set the active state to `null`
    pub fn clear_active_state(&self) {
        self.active_state.set(NullValue());
    }

    pub fn traverse_history(&self, direction: TraversalDirection) {
        let history_traversal_task_source = self.window.history_traversal_task_source();
        history_traversal_task_source.queue_history_traversal(&*self.window, direction);
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
        self.active_state.get()
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
