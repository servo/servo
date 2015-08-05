/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ScreenBinding;
use dom::bindings::codegen::Bindings::ScreenBinding::ScreenMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::window::Window;

#[dom_struct]
#[derive(HeapSizeOf)]
pub struct Screen {
    reflector_: Reflector,
}

impl Screen {
    fn new_inherited() -> Screen {
        Screen {
            reflector_: Reflector::new(),
        }
    }

    pub fn new(window: &Window) -> Root<Screen> {
        reflect_dom_object(box Screen::new_inherited(),
                           GlobalRef::Window(window),
                           ScreenBinding::Wrap)
    }
}

impl<'a> ScreenMethods for &'a Screen {
    // https://drafts.csswg.org/cssom-view/#dom-screen-colordepth
    fn ColorDepth(self) -> u32 {
        24
    }

    // https://drafts.csswg.org/cssom-view/#dom-screen-pixeldepth
    fn PixelDepth(self) -> u32 {
        24
    }
}

