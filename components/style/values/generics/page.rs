/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! @page at-rule properties

/// Page size names.
///
/// https://drafts.csswg.org/css-page-3/#typedef-page-size-page-size
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
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
    ToComputedValue,
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
#[derive(
    Clone,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
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
