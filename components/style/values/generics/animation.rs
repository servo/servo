/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic values for properties related to animations and transitions.

use crate::values::generics::length::GenericLengthPercentageOrAuto;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};

/// A generic value for the `[ [ auto | <length-percentage> ]{1,2} ]`.
///
/// https://drafts.csswg.org/scroll-animations-1/#view-timeline-inset
#[derive(
    Clone,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct GenericViewTimelineInset<LengthPercent> {
    /// The start inset in the relevant axis.
    pub start: GenericLengthPercentageOrAuto<LengthPercent>,
    /// The end inset.
    pub end: GenericLengthPercentageOrAuto<LengthPercent>,
}

pub use self::GenericViewTimelineInset as ViewTimelineInset;

impl<LengthPercent> ToCss for ViewTimelineInset<LengthPercent>
where
    LengthPercent: PartialEq + ToCss,
{
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        self.start.to_css(dest)?;
        if self.end != self.start {
            dest.write_char(' ')?;
            self.end.to_css(dest)?;
        }
        Ok(())
    }
}

impl<LengthPercent> Default for ViewTimelineInset<LengthPercent> {
    fn default() -> Self {
        Self {
            start: GenericLengthPercentageOrAuto::auto(),
            end: GenericLengthPercentageOrAuto::auto(),
        }
    }
}
