/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use malloc_size_of_derive::MallocSizeOf;

use super::Fragment;
use crate::geom::PhysicalRect;

/// A reference to a Fragment which is shared between `HoistedAbsolutelyPositionedBox`
/// and its placeholder `AbsoluteOrFixedPositionedFragment` in the original tree position.
/// This will be used later in order to paint this hoisted box in tree order.
#[derive(Default, MallocSizeOf)]
pub(crate) struct HoistedSharedFragment {
    pub fragment: Option<Fragment>,
    /// The original "static-position rect" of this absolutely positioned box. This is
    /// defined by the layout mode from which the box originates. As this fragment is
    /// hoisted up the tree this rectangle will be adjusted by the offsets of all
    /// ancestors between the tree position of the absolute and the containing block for
    /// absolutes that it is laid out in.
    ///
    /// See <https://drafts.csswg.org/css-position-3/#staticpos-rect>
    pub original_static_position_rect: PhysicalRect<Au>,
}

impl HoistedSharedFragment {
    pub(crate) fn new(original_static_position_rect: PhysicalRect<Au>) -> Self {
        Self {
            fragment: Default::default(),
            original_static_position_rect,
        }
    }
}
