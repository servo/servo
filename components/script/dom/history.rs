/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HistoryBinding::{self, HistoryMethods};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::window::{Window, WindowHelpers};

use msg::constellation_msg::{ConstellationChan, NavigationDirection};
use msg::constellation_msg::Msg as ConstellationMsg;

// https://developer.mozilla.org/en-US/docs/Web/API/History
#[dom_struct]
#[derive(HeapSizeOf)]
pub struct History {
    reflector_: Reflector,
    window: JS<Window>,
}

impl History {
    fn new_inherited(window: &Window) -> History {
        History {
            reflector_: Reflector::new(),
            window: JS::from_ref(window),
        }
    }

    pub fn new(window: &Window) -> Root<History> {
        reflect_dom_object(box History::new_inherited(window),
                           GlobalRef::Window(window),
                           HistoryBinding::Wrap)
    }
}

// https://html.spec.whatwg.org/multipage/#traverse-the-history-by-a-delta
fn traverse_history_by_delta(window: &Window, direction: NavigationDirection) {
    let ConstellationChan(ref chan) = window.constellation_chan();
    let msg = ConstellationMsg::Navigate(None, direction);
    chan.send(msg).unwrap();
}

impl<'a> HistoryMethods for &'a History {
    // https://html.spec.whatwg.org/multipage/#dom-history-back
    fn Back(self) {
        traverse_history_by_delta(&self.window.root(), NavigationDirection::Back)
    }

    // https://html.spec.whatwg.org/multipage/#dom-history-forward
    fn Forward(self) {
        traverse_history_by_delta(&self.window.root(), NavigationDirection::Forward)
    }
}
