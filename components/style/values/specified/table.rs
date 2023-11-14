/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified types for CSS values related to tables.

/// Specified values for the `caption-side` property.
///
/// Note that despite having "physical" names, these are actually interpreted
/// according to the table's writing-mode: Top and Bottom are treated as
/// block-start and -end respectively.
///
/// https://drafts.csswg.org/css-tables/#propdef-caption-side
#[allow(missing_docs)]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    FromPrimitive,
    MallocSizeOf,
    Ord,
    Parse,
    PartialEq,
    PartialOrd,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(u8)]
pub enum CaptionSide {
    Top,
    Bottom,
}
