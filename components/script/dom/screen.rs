/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ScreenBinding;
use dom::bindings::codegen::Bindings::ScreenBinding::ScreenMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::num::Finite;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::reflector::DomObject;
use dom::bindings::root::{Dom, DomRoot};
use dom::globalscope::GlobalScope;
use dom::window::Window;
use dom_struct::dom_struct;
use euclid::TypedSize2D;
use profile_traits::ipc;
use script_traits::ScriptMsg;
use style_traits::CSSPixel;
use typeholder::TypeHolderTrait;
use webrender_api::DeviceUintSize;

#[dom_struct]
pub struct Screen<TH: TypeHolderTrait> {
    reflector_: Reflector<TH>,
    window: Dom<Window<TH>>,
}

impl<TH: TypeHolderTrait> Screen<TH> {
    fn new_inherited(window: &Window<TH>) -> Screen<TH> {
        Screen {
            reflector_: Reflector::new(),
            window: Dom::from_ref(&window),
        }
    }

    pub fn new(window: &Window<TH>) -> DomRoot<Screen<TH>> {
        reflect_dom_object(Box::new(Screen::new_inherited(window)),
                           window,
                           ScreenBinding::Wrap)
    }

    fn screen_size(&self) -> TypedSize2D<u32, CSSPixel> {
        let (send, recv) = ipc::channel::<DeviceUintSize>(self.global().time_profiler_chan().clone()).unwrap();
        self.window.upcast::<GlobalScope<TH>>()
            .script_to_constellation_chan().send(ScriptMsg::GetScreenSize(send)).unwrap();
        let dpr = self.window.device_pixel_ratio();
        let screen = recv.recv().unwrap_or(TypedSize2D::zero());
        (screen.to_f32() / dpr).to_u32()
    }

    fn screen_avail_size(&self) -> TypedSize2D<u32, CSSPixel> {
        let (send, recv) = ipc::channel::<DeviceUintSize>(self.global().time_profiler_chan().clone()).unwrap();
        self.window.upcast::<GlobalScope<TH>>()
            .script_to_constellation_chan().send(ScriptMsg::GetScreenAvailSize(send)).unwrap();
        let dpr = self.window.device_pixel_ratio();
        let screen = recv.recv().unwrap_or(TypedSize2D::zero());
        (screen.to_f32() / dpr).to_u32()
    }
}

impl<TH: TypeHolderTrait> ScreenMethods for Screen<TH> {
    // https://drafts.csswg.org/cssom-view/#dom-screen-availwidth
    fn AvailWidth(&self) -> Finite<f64> {
        Finite::wrap(self.screen_avail_size().width as f64)
    }

    // https://drafts.csswg.org/cssom-view/#dom-screen-availheight
    fn AvailHeight(&self) -> Finite<f64> {
        Finite::wrap(self.screen_avail_size().height as f64)
    }

    // https://drafts.csswg.org/cssom-view/#dom-screen-width
    fn Width(&self) -> Finite<f64> {
        Finite::wrap(self.screen_size().width as f64)
    }

    // https://drafts.csswg.org/cssom-view/#dom-screen-height
    fn Height(&self) -> Finite<f64> {
        Finite::wrap(self.screen_size().height as f64)
    }

    // https://drafts.csswg.org/cssom-view/#dom-screen-colordepth
    fn ColorDepth(&self) -> u32 {
        24
    }

    // https://drafts.csswg.org/cssom-view/#dom-screen-pixeldepth
    fn PixelDepth(&self) -> u32 {
        24
    }
}
