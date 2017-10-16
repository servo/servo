/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HistoryBinding;
use dom::bindings::codegen::Bindings::HistoryBinding::HistoryMethods;
use dom::bindings::codegen::Bindings::LocationBinding::LocationBinding::LocationMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot};
use dom::globalscope::GlobalScope;
use dom::window::Window;
use dom_struct::dom_struct;
use ipc_channel::ipc;
use msg::constellation_msg::TraversalDirection;
use script_traits::ScriptMsg;

// https://html.spec.whatwg.org/multipage/#the-history-interface
#[dom_struct]
pub struct History {
    reflector_: Reflector,
    window: Dom<Window>,
}

impl History {
    pub fn new_inherited(window: &Window) -> History {
        History {
            reflector_: Reflector::new(),
            window: Dom::from_ref(&window),
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
}

impl HistoryMethods for History {
    // https://html.spec.whatwg.org/multipage/#dom-history-length
    fn GetLength(&self) -> Fallible<u32> {
        if !self.window.Document().is_fully_active() {
            return Err(Error::Security);
        }
        let (sender, recv) = ipc::channel().expect("Failed to create channel to send jsh length.");
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
}
