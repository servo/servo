/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HistoryBinding;
use dom::bindings::codegen::Bindings::HistoryBinding::HistoryMethods;
use dom::bindings::codegen::Bindings::LocationBinding::LocationMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::window::Window;
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
                           GlobalRef::Window(window),
                           HistoryBinding::Wrap)
    }
}

impl History {
    fn traverse_history(&self, direction: TraversalDirection) {
        let pipeline = self.window.pipeline();
        let msg = ConstellationMsg::TraverseHistory(Some(pipeline), direction);
        let _ = self.window.constellation_chan().send(msg);
    }
}

impl HistoryMethods for History {
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
}
