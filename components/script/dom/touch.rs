/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::TouchBinding::TouchMethods;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{DomRoot, MutDom};
use crate::dom::eventtarget::EventTarget;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct Touch {
    reflector_: Reflector,
    identifier: i32,
    target: MutDom<EventTarget>,
    screen_x: f64,
    screen_y: f64,
    client_x: f64,
    client_y: f64,
    page_x: f64,
    page_y: f64,
}

impl Touch {
    #[allow(clippy::too_many_arguments)]
    fn new_inherited(
        identifier: i32,
        target: &EventTarget,
        screen_x: Finite<f64>,
        screen_y: Finite<f64>,
        client_x: Finite<f64>,
        client_y: Finite<f64>,
        page_x: Finite<f64>,
        page_y: Finite<f64>,
    ) -> Touch {
        Touch {
            reflector_: Reflector::new(),
            identifier,
            target: MutDom::new(target),
            screen_x: *screen_x,
            screen_y: *screen_y,
            client_x: *client_x,
            client_y: *client_y,
            page_x: *page_x,
            page_y: *page_y,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        window: &Window,
        identifier: i32,
        target: &EventTarget,
        screen_x: Finite<f64>,
        screen_y: Finite<f64>,
        client_x: Finite<f64>,
        client_y: Finite<f64>,
        page_x: Finite<f64>,
        page_y: Finite<f64>,
        can_gc: CanGc,
    ) -> DomRoot<Touch> {
        reflect_dom_object(
            Box::new(Touch::new_inherited(
                identifier, target, screen_x, screen_y, client_x, client_y, page_x, page_y,
            )),
            window,
            can_gc,
        )
    }
}

impl TouchMethods<crate::DomTypeHolder> for Touch {
    /// <https://w3c.github.io/touch-events/#widl-Touch-identifier>
    fn Identifier(&self) -> i32 {
        self.identifier
    }

    /// <https://w3c.github.io/touch-events/#widl-Touch-target>
    fn Target(&self) -> DomRoot<EventTarget> {
        self.target.get()
    }

    /// <https://w3c.github.io/touch-events/#widl-Touch-screenX>
    fn ScreenX(&self) -> Finite<f64> {
        Finite::wrap(self.screen_x)
    }

    /// <https://w3c.github.io/touch-events/#widl-Touch-screenY>
    fn ScreenY(&self) -> Finite<f64> {
        Finite::wrap(self.screen_y)
    }

    /// <https://w3c.github.io/touch-events/#widl-Touch-clientX>
    fn ClientX(&self) -> Finite<f64> {
        Finite::wrap(self.client_x)
    }

    /// <https://w3c.github.io/touch-events/#widl-Touch-clientY>
    fn ClientY(&self) -> Finite<f64> {
        Finite::wrap(self.client_y)
    }

    /// <https://w3c.github.io/touch-events/#widl-Touch-clientX>
    fn PageX(&self) -> Finite<f64> {
        Finite::wrap(self.page_x)
    }

    /// <https://w3c.github.io/touch-events/#widl-Touch-clientY>
    fn PageY(&self) -> Finite<f64> {
        Finite::wrap(self.page_y)
    }
}
