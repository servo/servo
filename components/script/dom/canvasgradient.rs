/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::canvas::{CanvasGradientStop, FillOrStrokeStyle, LinearGradientStyle, RadialGradientStyle};
use cssparser::{Parser, ParserInput, RGBA};
use cssparser::Color as CSSColor;
use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::CanvasGradientBinding;
use dom::bindings::codegen::Bindings::CanvasGradientBinding::CanvasGradientMethods;
use dom::bindings::error::{Error, ErrorResult};
use dom::bindings::num::Finite;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;

// https://html.spec.whatwg.org/multipage/#canvasgradient
#[dom_struct]
pub struct CanvasGradient {
    reflector_: Reflector,
    style: CanvasGradientStyle,
    stops: DomRefCell<Vec<CanvasGradientStop>>,
}

#[derive(Clone, HeapSizeOf, JSTraceable)]
pub enum CanvasGradientStyle {
    Linear(LinearGradientStyle),
    Radial(RadialGradientStyle),
}

impl CanvasGradient {
    fn new_inherited(style: CanvasGradientStyle) -> CanvasGradient {
        CanvasGradient {
            reflector_: Reflector::new(),
            style: style,
            stops: DomRefCell::new(Vec::new()),
        }
    }

    pub fn new(global: &GlobalScope, style: CanvasGradientStyle) -> DomRoot<CanvasGradient> {
        reflect_dom_object(Box::new(CanvasGradient::new_inherited(style)),
                           global,
                           CanvasGradientBinding::Wrap)
    }
}

impl CanvasGradientMethods for CanvasGradient {
    // https://html.spec.whatwg.org/multipage/#dom-canvasgradient-addcolorstop
    fn AddColorStop(&self, offset: Finite<f64>, color: DOMString) -> ErrorResult {
        if *offset < 0f64 || *offset > 1f64 {
            return Err(Error::IndexSize);
        }

        let mut input = ParserInput::new(&color);
        let mut parser = Parser::new(&mut input);
        let color = CSSColor::parse(&mut parser);
        let color = if parser.is_exhausted() {
            match color {
                Ok(CSSColor::RGBA(rgba)) => rgba,
                Ok(CSSColor::CurrentColor) => RGBA::new(0, 0, 0, 255),
                _ => return Err(Error::Syntax)
            }
        } else {
            return Err(Error::Syntax)
        };

        self.stops.borrow_mut().push(CanvasGradientStop {
            offset: (*offset) as f64,
            color: color,
        });
        Ok(())
    }
}

pub trait ToFillOrStrokeStyle {
    fn to_fill_or_stroke_style(self) -> FillOrStrokeStyle;
}

impl<'a> ToFillOrStrokeStyle for &'a CanvasGradient {
    fn to_fill_or_stroke_style(self) -> FillOrStrokeStyle {
        let gradient_stops = self.stops.borrow().clone();
        match self.style {
            CanvasGradientStyle::Linear(ref gradient) => {
                FillOrStrokeStyle::LinearGradient(LinearGradientStyle::new(gradient.x0,
                                                                           gradient.y0,
                                                                           gradient.x1,
                                                                           gradient.y1,
                                                                           gradient_stops))
            }
            CanvasGradientStyle::Radial(ref gradient) => {
                FillOrStrokeStyle::RadialGradient(RadialGradientStyle::new(gradient.x0,
                                                                           gradient.y0,
                                                                           gradient.r0,
                                                                           gradient.x1,
                                                                           gradient.y1,
                                                                           gradient.r1,
                                                                           gradient_stops))
            }
        }
    }
}
