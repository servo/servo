/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Computed @page at-rule properties and named-page style properties

use crate::values::computed::length::NonNegativeLength;
use crate::values::computed::{Context, ToComputedValue};
use crate::values::generics;
use crate::values::generics::size::Size2D;

use crate::values::specified::page as specified;
pub use generics::page::GenericPageSize;
pub use generics::page::PageOrientation;
pub use generics::page::PageSizeOrientation;
pub use generics::page::PaperSize;
pub use specified::PageName;

/// Computed value of the @page size descriptor
///
/// The spec says that the computed value should be the same as the specified
/// value but with all absolute units, but it's not currently possibly observe
/// the computed value of page-size.
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToCss, ToResolvedValue, ToShmem)]
#[repr(C, u8)]
pub enum PageSize {
    /// Specified size, paper size, or paper size and orientation.
    Size(Size2D<NonNegativeLength>),
    /// `landscape` or `portrait` value, no specified size.
    Orientation(PageSizeOrientation),
    /// `auto` value
    Auto,
}

impl ToComputedValue for specified::PageSize {
    type ComputedValue = PageSize;

    fn to_computed_value(&self, ctx: &Context) -> Self::ComputedValue {
        match &*self {
            Self::Size(s) => PageSize::Size(s.to_computed_value(ctx)),
            Self::PaperSize(p, PageSizeOrientation::Landscape) => PageSize::Size(Size2D {
                width: p.long_edge().to_computed_value(ctx),
                height: p.short_edge().to_computed_value(ctx),
            }),
            Self::PaperSize(p, PageSizeOrientation::Portrait) => PageSize::Size(Size2D {
                width: p.short_edge().to_computed_value(ctx),
                height: p.long_edge().to_computed_value(ctx),
            }),
            Self::Orientation(o) => PageSize::Orientation(*o),
            Self::Auto => PageSize::Auto,
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            PageSize::Size(s) => Self::Size(ToComputedValue::from_computed_value(&s)),
            PageSize::Orientation(o) => Self::Orientation(o),
            PageSize::Auto => Self::Auto,
        }
    }
}

impl PageSize {
    /// `auto` value.
    #[inline]
    pub fn auto() -> Self {
        PageSize::Auto
    }

    /// Whether this is the `auto` value.
    #[inline]
    pub fn is_auto(&self) -> bool {
        matches!(*self, PageSize::Auto)
    }
}
