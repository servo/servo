/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use canvas_traits::canvas::{
    CanvasGradientStop, FillOrStrokeStyle, LinearGradientStyle, RadialGradientStyle,
};
use dom_struct::dom_struct;

use crate::canvas_state::parse_color;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasGradientMethods;
use crate::dom::bindings::error::{Error, ErrorResult};
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

// https://html.spec.whatwg.org/multipage/#canvasgradient
#[dom_struct]
pub(crate) struct CanvasGradient {
    reflector_: Reflector,
    style: CanvasGradientStyle,
    #[no_trace]
    stops: DomRefCell<Vec<CanvasGradientStop>>,
}

#[derive(Clone, JSTraceable, MallocSizeOf)]
pub(crate) enum CanvasGradientStyle {
    Linear(#[no_trace] LinearGradientStyle),
    Radial(#[no_trace] RadialGradientStyle),
}

impl CanvasGradient {
    fn new_inherited(style: CanvasGradientStyle) -> CanvasGradient {
        CanvasGradient {
            reflector_: Reflector::new(),
            style,
            stops: DomRefCell::new(Vec::new()),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        style: CanvasGradientStyle,
        can_gc: CanGc,
    ) -> DomRoot<CanvasGradient> {
        reflect_dom_object(
            Box::new(CanvasGradient::new_inherited(style)),
            global,
            can_gc,
        )
    }
}

impl CanvasGradientMethods<crate::DomTypeHolder> for CanvasGradient {
    // https://html.spec.whatwg.org/multipage/#dom-canvasgradient-addcolorstop
    fn AddColorStop(&self, offset: Finite<f64>, color: DOMString, can_gc: CanGc) -> ErrorResult {
        if *offset < 0f64 || *offset > 1f64 {
            return Err(Error::IndexSize);
        }

        let color = match parse_color(None, &color, can_gc) {
            Ok(color) => color,
            Err(_) => return Err(Error::Syntax),
        };

        self.stops.borrow_mut().push(CanvasGradientStop {
            offset: (*offset),
            color,
        });
        Ok(())
    }
}

pub(crate) trait ToFillOrStrokeStyle {
    fn to_fill_or_stroke_style(self) -> FillOrStrokeStyle;
}

impl ToFillOrStrokeStyle for &CanvasGradient {
    fn to_fill_or_stroke_style(self) -> FillOrStrokeStyle {
        let gradient_stops = self.stops.borrow().clone();
        match self.style {
            CanvasGradientStyle::Linear(ref gradient) => {
                FillOrStrokeStyle::LinearGradient(LinearGradientStyle::new(
                    gradient.x0,
                    gradient.y0,
                    gradient.x1,
                    gradient.y1,
                    gradient_stops,
                ))
            },
            CanvasGradientStyle::Radial(ref gradient) => {
                FillOrStrokeStyle::RadialGradient(RadialGradientStyle::new(
                    gradient.x0,
                    gradient.y0,
                    gradient.r0,
                    gradient.x1,
                    gradient.y1,
                    gradient.r1,
                    gradient_stops,
                ))
            },
        }
    }
}
