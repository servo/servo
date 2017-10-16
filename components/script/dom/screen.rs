/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ScreenBinding;
use dom::bindings::codegen::Bindings::ScreenBinding::ScreenMethods;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::window::Window;
use dom_struct::dom_struct;

#[dom_struct]
pub struct Screen {
    reflector_: Reflector,
}

impl Screen {
    fn new_inherited() -> Screen {
        Screen {
            reflector_: Reflector::new(),
        }
    }

    pub fn new(window: &Window) -> DomRoot<Screen> {
        reflect_dom_object(Box::new(Screen::new_inherited()),
                           window,
                           ScreenBinding::Wrap)
    }
}

impl ScreenMethods for Screen {
    // https://drafts.csswg.org/cssom-view/#dom-screen-colordepth
    fn ColorDepth(&self) -> u32 {
        24
    }

    // https://drafts.csswg.org/cssom-view/#dom-screen-pixeldepth
    fn PixelDepth(&self) -> u32 {
        24
    }
}
