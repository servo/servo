/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed values for inherited box

use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};
use values::specified::Angle;

/// An angle rounded and normalized per https://drafts.csswg.org/css-images/#propdef-image-orientation
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq)]
pub enum Orientation {
    Angle0 = 0,
    Angle90,
    Angle180,
    Angle270,
}

impl Orientation {
    /// Get the actual angle that this orientation value represents.
    pub fn angle(&self) -> Angle {
        match *self {
            Orientation::Angle0 => Angle::from_degrees(0.0, false),
            Orientation::Angle90 => Angle::from_degrees(90.0, false),
            Orientation::Angle180 => Angle::from_degrees(180.0, false),
            Orientation::Angle270 => Angle::from_degrees(270.0, false),
        }
    }
}

impl ToCss for Orientation {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        // Should agree with Angle::to_css.
        match *self {
            Orientation::Angle0 => dest.write_str("0deg"),
            Orientation::Angle90 => dest.write_str("90deg"),
            Orientation::Angle180 => dest.write_str("180deg"),
            Orientation::Angle270 => dest.write_str("270deg"),
        }
    }
}

/// https://drafts.csswg.org/css-images/#propdef-image-orientation
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq)]
pub enum ImageOrientation {
    /// 'from-image'
    FromImage,

    /// '<angle>' | '<angle>? flip'
    AngleWithFlipped(Orientation, bool),
}

impl ImageOrientation {
    #[allow(missing_docs)]
    pub fn zero() -> Self {
        ImageOrientation::AngleWithFlipped(Orientation::Angle0, false)
    }
}

impl ToCss for ImageOrientation {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match *self {
            ImageOrientation::FromImage => dest.write_str("from-image"),
            ImageOrientation::AngleWithFlipped(angle, flipped) => {
                angle.to_css(dest)?;
                if flipped {
                    dest.write_str(" flip")?;
                }
                Ok(())
            },
        }
    }
}
