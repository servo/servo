/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::TouchBinding;
use dom::bindings::codegen::Bindings::TouchBinding::TouchMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, MutHeap, Root};
use dom::bindings::num::Finite;
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::eventtarget::EventTarget;
use dom::window::Window;

#[dom_struct]
pub struct Touch {
    reflector_: Reflector,
    identifier: i32,
    target: MutHeap<JS<EventTarget>>,
    screen_x: f64,
    screen_y: f64,
    client_x: f64,
    client_y: f64,
    page_x: f64,
    page_y: f64,
}

impl Touch {
    fn new_inherited(identifier: i32, target: &EventTarget,
                     screen_x: Finite<f64>, screen_y: Finite<f64>,
                     client_x: Finite<f64>, client_y: Finite<f64>,
                     page_x: Finite<f64>, page_y: Finite<f64>) -> Touch {
        Touch {
            reflector_: Reflector::new(),
            identifier: identifier,
            target: MutHeap::new(target),
            screen_x: *screen_x,
            screen_y: *screen_y,
            client_x: *client_x,
            client_y: *client_y,
            page_x: *page_x,
            page_y: *page_y,
        }
    }

    pub fn new(window: &Window, identifier: i32, target: &EventTarget,
              screen_x: Finite<f64>, screen_y: Finite<f64>,
              client_x: Finite<f64>, client_y: Finite<f64>,
              page_x: Finite<f64>, page_y: Finite<f64>) -> Root<Touch> {
        reflect_dom_object(box Touch::new_inherited(identifier, target,
                                                    screen_x, screen_y,
                                                    client_x, client_y,
                                                    page_x, page_y),
                           GlobalRef::Window(window), TouchBinding::Wrap)
    }
}

impl TouchMethods for Touch {
    /// https://w3c.github.io/touch-events/#widl-Touch-identifier
    fn Identifier(&self) -> i32 {
        self.identifier
    }

    /// https://w3c.github.io/touch-events/#widl-Touch-target
    fn Target(&self) -> Root<EventTarget> {
        self.target.get()
    }

    /// https://w3c.github.io/touch-events/#widl-Touch-screenX
    fn ScreenX(&self) -> Finite<f64> {
        Finite::wrap(self.screen_x)
    }

    /// https://w3c.github.io/touch-events/#widl-Touch-screenY
    fn ScreenY(&self) -> Finite<f64> {
        Finite::wrap(self.screen_y)
    }

    /// https://w3c.github.io/touch-events/#widl-Touch-clientX
    fn ClientX(&self) -> Finite<f64> {
        Finite::wrap(self.client_x)
    }

    /// https://w3c.github.io/touch-events/#widl-Touch-clientY
    fn ClientY(&self) -> Finite<f64> {
        Finite::wrap(self.client_y)
    }

    /// https://w3c.github.io/touch-events/#widl-Touch-clientX
    fn PageX(&self) -> Finite<f64> {
        Finite::wrap(self.page_x)
    }

    /// https://w3c.github.io/touch-events/#widl-Touch-clientY
    fn PageY(&self) -> Finite<f64> {
        Finite::wrap(self.page_y)
    }
}
