/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use canvas_traits::canvas::{FillOrStrokeStyle, RepetitionStyle, SurfaceStyle};
use dom_struct::dom_struct;
use euclid::default::Size2D;

use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::canvasgradient::ToFillOrStrokeStyle;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

// https://html.spec.whatwg.org/multipage/#canvaspattern
#[dom_struct]
pub(crate) struct CanvasPattern {
    reflector_: Reflector,
    surface_data: Vec<u8>,
    #[no_trace]
    surface_size: Size2D<u32>,
    repeat_x: bool,
    repeat_y: bool,
    origin_clean: bool,
}

impl CanvasPattern {
    fn new_inherited(
        surface_data: Vec<u8>,
        surface_size: Size2D<u32>,
        repeat: RepetitionStyle,
        origin_clean: bool,
    ) -> CanvasPattern {
        let (x, y) = match repeat {
            RepetitionStyle::Repeat => (true, true),
            RepetitionStyle::RepeatX => (true, false),
            RepetitionStyle::RepeatY => (false, true),
            RepetitionStyle::NoRepeat => (false, false),
        };

        CanvasPattern {
            reflector_: Reflector::new(),
            surface_data,
            surface_size,
            repeat_x: x,
            repeat_y: y,
            origin_clean,
        }
    }
    pub(crate) fn new(
        global: &GlobalScope,
        surface_data: Vec<u8>,
        surface_size: Size2D<u32>,
        repeat: RepetitionStyle,
        origin_clean: bool,
        can_gc: CanGc,
    ) -> DomRoot<CanvasPattern> {
        reflect_dom_object(
            Box::new(CanvasPattern::new_inherited(
                surface_data,
                surface_size,
                repeat,
                origin_clean,
            )),
            global,
            can_gc,
        )
    }
    pub(crate) fn origin_is_clean(&self) -> bool {
        self.origin_clean
    }
}

impl ToFillOrStrokeStyle for &CanvasPattern {
    fn to_fill_or_stroke_style(self) -> FillOrStrokeStyle {
        FillOrStrokeStyle::Surface(SurfaceStyle::new(
            self.surface_data.clone(),
            self.surface_size,
            self.repeat_x,
            self.repeat_y,
        ))
    }
}
