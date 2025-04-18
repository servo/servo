/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use malloc_size_of_derive::MallocSizeOf;
use style::logical_geometry::WritingMode;
use style::values::specified::align::AlignFlags;

use super::Fragment;
use crate::geom::{LogicalVec2, PhysicalRect, PhysicalVec};

/// A reference to a Fragment which is shared between `HoistedAbsolutelyPositionedBox`
/// and its placeholder `AbsoluteOrFixedPositionedFragment` in the original tree position.
/// This will be used later in order to paint this hoisted box in tree order.
#[derive(MallocSizeOf)]
pub(crate) struct HoistedSharedFragment {
    pub fragment: Option<Fragment>,
    /// The "static-position rect" of this absolutely positioned box. This is defined by the
    /// layout mode from which the box originates.
    ///
    /// See <https://drafts.csswg.org/css-position-3/#staticpos-rect>
    pub static_position_rect: PhysicalRect<Au>,
    /// The resolved alignment values used for aligning this absolutely positioned element
    /// if the "static-position rect" ends up being the "inset-modified containing block".
    /// These values are dependent on the layout mode (currently only interesting for
    /// flexbox).
    pub resolved_alignment: LogicalVec2<AlignFlags>,
    /// This is the [`WritingMode`] of the original parent of the element that created this
    /// hoisted absolutely-positioned fragment. This helps to interpret the offset for
    /// static positioning. If the writing mode is right-to-left or bottom-to-top, the static
    /// offset needs to be adjusted by the absolutely positioned element's inline size.
    pub original_parent_writing_mode: WritingMode,
}

impl HoistedSharedFragment {
    pub(crate) fn new(
        static_position_rect: PhysicalRect<Au>,
        resolved_alignment: LogicalVec2<AlignFlags>,
        original_parent_writing_mode: WritingMode,
    ) -> Self {
        HoistedSharedFragment {
            fragment: None,
            static_position_rect,
            resolved_alignment,
            original_parent_writing_mode,
        }
    }
}

impl HoistedSharedFragment {
    /// `inset: auto`-positioned elements do not know their precise position until after
    /// they're hoisted. This lets us adjust auto values after the fact.
    pub(crate) fn adjust_offsets(&mut self, offset: &PhysicalVec<Au>) {
        self.static_position_rect = self.static_position_rect.translate(*offset);
    }
}
