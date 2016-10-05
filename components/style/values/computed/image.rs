/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the computed value of
//! [`image`][image]s
//!
//! [image]: https://drafts.csswg.org/css-images/#image-values

use cssparser::Color as CSSColor;
use std::fmt;
use std::fmt::Debug;
use url::Url;
use values::LocalToCss;
use values::computed::{Context, Length, LengthOrPercentage, ToComputedValue};
use values::specified;
use values::specified::UrlExtraData;
use values::specified::image::{GradientKind, SizeKeyword};

impl ToComputedValue for specified::image::Image {
    type ComputedValue = Image;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Image {
        match *self {
            specified::image::Image::Url(ref url, ref extra_data) => {
                Image::Url(url.clone(), extra_data.clone())
            },
            specified::image::Image::Gradient(ref gradient) => {
                Image::Gradient(gradient.to_computed_value(context))
            }
        }
    }

    #[inline]
    fn from_computed_value(computed: &Image) -> Self {
        match *computed {
            Image::Url(ref url, ref extra_data) => {
                specified::image::Image::Url(url.clone(), extra_data.clone())
            },
            Image::Gradient(ref linear_gradient) => {
                specified::image::Image::Gradient(
                    ToComputedValue::from_computed_value(linear_gradient)
                )
            }
        }
    }
}

/// Computed values for an image according to CSS-IMAGES.
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum Image {
    Url(Url, UrlExtraData),
    Gradient(Gradient),
}

impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Image::Url(ref url, ref _extra_data) => write!(f, "url(\"{}\")", url),
            Image::Gradient(ref grad) => {
                if grad.repeating {
                    let _ = write!(f, "repeating-");
                }
                match grad.gradient_kind {
                    GradientKind::Linear(_) => write!(f, "linear-gradient({:?})", grad),
                    GradientKind::Radial(_, _) => write!(f, "radial-gradient({:?})", grad),
                }
            },
        }
    }
}

impl ::cssparser::ToCss for Image {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        use values::LocalToCss;
        match *self {
            Image::Url(ref url, _) => {
                url.to_css(dest)
            }
            Image::Gradient(ref gradient) => gradient.to_css(dest)
        }
    }
}

/// Computed values for a CSS gradient.
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Gradient {
    /// The color stops.
    pub stops: Vec<ColorStop>,
    /// True if this is a repeating gradient.
    pub repeating: bool,
    /// Gradient kind can be linear or radial.
    pub gradient_kind: GradientKind,
}

impl ::cssparser::ToCss for Gradient {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        if self.repeating {
            try!(dest.write_str("repeating-"));
        }
        match self.gradient_kind {
            GradientKind::Linear(angle_or_corner) => {
                try!(dest.write_str("linear-gradient("));
                try!(angle_or_corner.to_css(dest));
            },
            GradientKind::Radial(ref shape, position) => {
                try!(dest.write_str("radial-gradient("));
                try!(shape.to_css(dest));
                try!(dest.write_str(" at "));
                try!(position.to_css(dest));
            },
        }
        for stop in &self.stops {
            try!(dest.write_str(", "));
            try!(stop.to_css(dest));
        }
        try!(dest.write_str(")"));
        Ok(())
    }
}

impl fmt::Debug for Gradient {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.gradient_kind {
            GradientKind::Linear(angle_or_corner) => {
                let _ = write!(f, "{:?}", angle_or_corner);
            },
            GradientKind::Radial(ref shape, position) => {
                let _ = write!(f, "{:?} at {:?}", shape, position);
            },
        }

        for stop in &self.stops {
            let _ = write!(f, ", {:?}", stop);
        }
        Ok(())
    }
}

/// Computed values for one color stop in a linear gradient.
#[derive(Clone, PartialEq, Copy)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct ColorStop {
    /// The color of this stop.
    pub color: CSSColor,

    /// The position of this stop. If not specified, this stop is placed halfway between the
    /// point that precedes it and the point that follows it per CSS-IMAGES § 3.4.
    pub position: Option<LengthOrPercentage>,
}

impl ::cssparser::ToCss for ColorStop {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(self.color.to_css(dest));
        if let Some(position) = self.position {
            try!(dest.write_str(" "));
            try!(position.to_css(dest));
        }
        Ok(())
    }
}

impl fmt::Debug for ColorStop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let _ = write!(f, "{:?}", self.color);
        self.position.map(|pos| {
            let _ = write!(f, " {:?}", pos);
        });
        Ok(())
    }
}

/// Computed values for EndingShape
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum EndingShape {
    Circle(Size<Length>),
    Ellipse(Size<Length>, Size<Length>),
}

impl ::cssparser::ToCss for EndingShape {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            EndingShape::Circle(ref length) => {
                try!(dest.write_str("circle "));
                try!(length.to_css(dest));
            },
            EndingShape::Ellipse(ref first_len, ref second_len) => {
                try!(dest.write_str("ellipse "));
                try!(first_len.to_css(dest));
                try!(dest.write_str(" "));
                try!(second_len.to_css(dest));
            },
        }
        Ok(())
    }
}

impl fmt::Debug for EndingShape {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EndingShape::Circle(ref length) => {
                let _ = write!(f, "circle {:?}", length);
            },
            EndingShape::Ellipse(ref first_len, ref second_len) => {
                let _ = write!(f, "ellipse {:?} {:?}", first_len, second_len);
            }
        }
        Ok(())
    }
}

#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum Size<T: Debug + LocalToCss> {
    Length(T),
    Keyword(SizeKeyword),
}

impl<T: Debug + LocalToCss> ::cssparser::ToCss for Size<T> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            Size::Length(ref length) => length.to_css(dest),
            Size::Keyword(keyword) => keyword.to_css(dest),
        }
    }
}

impl<T: Debug + LocalToCss> fmt::Debug for Size<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Size::Length(ref length) => {
                let _ = write!(f, "{:?}", length);
            },
            Size::Keyword(keyword) => {
                let _ = write!(f, "{:?}", keyword);
            },
        }
        Ok(())
    }
}

impl ToComputedValue for specified::image::Gradient {
    type ComputedValue = Gradient;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Gradient {
        let specified::image::Gradient {
            ref stops,
            repeating,
            ref gradient_kind
        } = *self;
        Gradient {
            stops: stops.iter().map(|stop| {
                ColorStop {
                    color: stop.color.parsed,
                    position: match stop.position {
                        None => None,
                        Some(value) => Some(value.to_computed_value(context)),
                    },
                }
            }).collect(),
            repeating: repeating,
            gradient_kind: gradient_kind.clone()
        }
    }
    #[inline]
    fn from_computed_value(computed: &Gradient) -> Self {
        let Gradient {
            ref stops,
            repeating,
            ref gradient_kind
        } = *computed;
        specified::image::Gradient {
            stops: stops.iter().map(|stop| {
                specified::image::ColorStop {
                    color: ToComputedValue::from_computed_value(&stop.color),
                    position: match stop.position {
                        None => None,
                        Some(value) => Some(ToComputedValue::from_computed_value(&value)),
                    },
                }
            }).collect(),
            repeating: repeating,
            gradient_kind: gradient_kind.clone()
        }
    }
}
