/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use euclid::Size2D;
use profile_traits::ipc;
use servo_geometry::DeviceIndependentIntSize;
use style_traits::CSSPixel;
use webrender_traits::CrossProcessCompositorMessage;

use crate::dom::bindings::codegen::Bindings::ScreenBinding::ScreenMethods;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct Screen {
    reflector_: Reflector,
    window: Dom<Window>,
}

impl Screen {
    fn new_inherited(window: &Window) -> Screen {
        Screen {
            reflector_: Reflector::new(),
            window: Dom::from_ref(window),
        }
    }

    pub(crate) fn new(window: &Window) -> DomRoot<Screen> {
        reflect_dom_object(
            Box::new(Screen::new_inherited(window)),
            window,
            CanGc::note(),
        )
    }

    fn screen_size(&self) -> Size2D<u32, CSSPixel> {
        let (send, recv) =
            ipc::channel::<DeviceIndependentIntSize>(self.global().time_profiler_chan().clone())
                .unwrap();
        self.window
            .compositor_api()
            .sender()
            .send(CrossProcessCompositorMessage::GetScreenSize(send))
            .unwrap();
        let size = recv.recv().unwrap_or(Size2D::zero()).to_u32();
        Size2D::new(size.width, size.height)
    }

    fn screen_avail_size(&self) -> Size2D<u32, CSSPixel> {
        let (send, recv) =
            ipc::channel::<DeviceIndependentIntSize>(self.global().time_profiler_chan().clone())
                .unwrap();
        self.window
            .compositor_api()
            .sender()
            .send(CrossProcessCompositorMessage::GetAvailableScreenSize(send))
            .unwrap();
        let size = recv.recv().unwrap_or(Size2D::zero()).to_u32();
        Size2D::new(size.width, size.height)
    }
}

impl ScreenMethods<crate::DomTypeHolder> for Screen {
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
