/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ScreenBinding;
use dom::bindings::codegen::Bindings::ScreenBinding::ScreenMethods;
use dom::bindings::global;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::window::Window;

#[jstraceable]
#[must_root]
pub struct Screen {
    reflector_: Reflector,
}

impl Screen {
    fn new_inherited() -> Screen {
        Screen {
            reflector_: Reflector::new(),
        }
    }

    pub fn new(window: JSRef<Window>) -> Temporary<Screen> {
        reflect_dom_object(box Screen::new_inherited(),
                           &global::Window(window),
                           ScreenBinding::Wrap)
    }
}

impl<'a> ScreenMethods for JSRef<'a, Screen> {
    fn ColorDepth(self) -> u32 {
        24
    }

    fn PixelDepth(self) -> u32 {
        24
    }
}

impl Reflectable for Screen {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
