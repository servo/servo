/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! @page at-rule properties

use crate::values::generics::NonNegative;
use crate::values::specified::length::AbsoluteLength;

/// Page size names.
///
/// https://drafts.csswg.org/css-page-3/#typedef-page-size-page-size
#[derive(
    Clone, Copy, Debug, Eq, MallocSizeOf, Parse, PartialEq, SpecifiedValueInfo, ToCss, ToShmem,
)]
#[repr(u8)]
pub enum PaperSize {
    /// ISO A5 media
    A5,
    /// ISO A4 media
    A4,
    /// ISO A3 media
    A3,
    /// ISO B5 media
    B5,
    /// ISO B4 media
    B4,
    /// JIS B5 media
    JisB5,
    /// JIS B4 media
    JisB4,
    /// North American Letter size
    Letter,
    /// North American Legal size
    Legal,
    /// North American Ledger size
    Ledger,
}

impl PaperSize {
    /// Gets the long edge length of the paper size
    pub fn long_edge(&self) -> NonNegative<AbsoluteLength> {
        NonNegative(match *self {
            PaperSize::A5 => AbsoluteLength::Mm(210.0),
            PaperSize::A4 => AbsoluteLength::Mm(297.0),
            PaperSize::A3 => AbsoluteLength::Mm(420.0),
            PaperSize::B5 => AbsoluteLength::Mm(250.0),
            PaperSize::B4 => AbsoluteLength::Mm(353.0),
            PaperSize::JisB5 => AbsoluteLength::Mm(257.0),
            PaperSize::JisB4 => AbsoluteLength::Mm(364.0),
            PaperSize::Letter => AbsoluteLength::In(11.0),
            PaperSize::Legal => AbsoluteLength::In(14.0),
            PaperSize::Ledger => AbsoluteLength::In(17.0),
        })
    }
    /// Gets the short edge length of the paper size
    pub fn short_edge(&self) -> NonNegative<AbsoluteLength> {
        NonNegative(match *self {
            PaperSize::A5 => AbsoluteLength::Mm(148.0),
            PaperSize::A4 => AbsoluteLength::Mm(210.0),
            PaperSize::A3 => AbsoluteLength::Mm(297.0),
            PaperSize::B5 => AbsoluteLength::Mm(176.0),
            PaperSize::B4 => AbsoluteLength::Mm(250.0),
            PaperSize::JisB5 => AbsoluteLength::Mm(182.0),
            PaperSize::JisB4 => AbsoluteLength::Mm(257.0),
            PaperSize::Letter => AbsoluteLength::In(8.5),
            PaperSize::Legal => AbsoluteLength::In(8.5),
            PaperSize::Ledger => AbsoluteLength::In(11.0),
        })
    }
}

/// Paper orientation
///
/// https://drafts.csswg.org/css-page-3/#page-size-prop
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(u8)]
pub enum Orientation {
    /// Portrait orientation
    Portrait,
    /// Landscape orientation
    Landscape,
}

/// Page size property
///
/// https://drafts.csswg.org/css-page-3/#page-size-prop
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
#[repr(C, u8)]
pub enum GenericPageSize<S> {
    /// Page dimensions.
    Size(S),
    /// Paper size with no orientation.
    PaperSize(PaperSize),
    /// An orientation with no size.
    Orientation(Orientation),
    /// Paper size by name, with an orientation.
    PaperSizeAndOrientation(PaperSize, Orientation),
    /// `auto` value.
    Auto,
}

pub use self::GenericPageSize as PageSize;

impl<S> PageSize<S> {
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
