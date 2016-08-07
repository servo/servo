/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HistoryBinding;
use dom::bindings::codegen::Bindings::HistoryBinding::HistoryMethods;
use dom::bindings::codegen::Bindings::LocationBinding::LocationMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::error::{Error, ErrorResult};
use dom::bindings::structuredclone::StructuredCloneData;
use dom::bindings::globalscope::GlobalScope;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::str::{DOMString, USVString};
use dom::urlhelper::UrlHelper;
use dom::window::Window;
use ipc_channel::ipc;
use js::jsapi::{HandleValue, JSContext};
use js::jsval::{JSVal, NullValue};
use msg::constellation_msg::TraversalDirection;
use script_traits::ScriptMsg as ConstellationMsg;

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
                           window,
                           HistoryBinding::Wrap)
    }

    fn add_state(&self,
                 cx: *mut JSContext,
                 data: HandleValue,
                 title: DOMString,
                 url: Option<USVString>,
                 replace: bool) -> ErrorResult {
        // Step 1
        let document = self.window.Document();
        // Step 2
        if !document.is_fully_active() {
            return Err(Error::Security);
        }
        // Step 5
        let cloned_data = try!(StructuredCloneData::write(cx, data));
        rooted!(in(cx) let mut state = NullValue());
        cloned_data.read(GlobalRef::Window(&*self.window), state.handle_mut());
        let url = match url {
            Some(url) => {
                // Step 6
                let document_url = document.url();
                // 6.1
                let url = match document_url.join(&url.0) {
                    Ok(url) => url,
                    // 6.2
                    Err(_) => return Err(Error::Security),
                };

                // 6.4
                if url.scheme() != document_url.scheme() ||
                   url.username() != document_url.username() ||
                   url.password() != document_url.password() ||
                   url.host() != document_url.host() ||
                   url.port() != document_url.port() {
                    return Err(Error::Security);
                }

                // 6.5
                if !UrlHelper::SameOrigin(&url, document_url) &&
                   (url.path() != document_url.path() || url.query() != document_url.query()) {
                    return Err(Error::Security);
                }

                url
            },
            // Step 7
            None => document.url().clone(),
        };

        // Step 8
        if replace {
            self.window.browsing_context().replace_session_history_entry(Some(title), Some(url), state.handle());
        } else {
            self.window.browsing_context().push_session_history_entry(&*document, Some(title), Some(url), Some(state.handle()));
        }

        Ok(())
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

    // https://html.spec.whatwg.org/multipage/#dom-history-state
    fn State(&self, _cx: *mut JSContext) -> JSVal {
        self.window.browsing_context().state()
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

    // https://html.spec.whatwg.org/multipage/#dom-history-pushtstae
    fn PushState(&self,
                 cx: *mut JSContext,
                 data: HandleValue,
                 title: DOMString,
                 url: Option<USVString>) -> ErrorResult {
        self.add_state(cx, data, title, url, false)
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-replacestate
    fn ReplaceState(&self,
                    cx: *mut JSContext,
                    data: HandleValue,
                    title: DOMString,
                    url: Option<USVString>) -> ErrorResult {
        self.add_state(cx, data, title, url, true)
    }
}
