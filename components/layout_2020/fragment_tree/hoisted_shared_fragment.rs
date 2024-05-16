/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use app_units::Au;
use serde::Serialize;
use style::values::computed::LengthPercentage;

use super::Fragment;
use crate::cell::ArcRefCell;
use crate::geom::LogicalVec2;

/// A reference to a Fragment which is shared between `HoistedAbsolutelyPositionedBox`
/// and its placeholder `AbsoluteOrFixedPositionedFragment` in the original tree position.
/// This will be used later in order to paint this hoisted box in tree order.
#[derive(Serialize)]
pub(crate) struct HoistedSharedFragment {
    pub fragment: Option<ArcRefCell<Fragment>>,
    pub box_offsets: LogicalVec2<AbsoluteBoxOffsets>,
}

impl HoistedSharedFragment {
    pub(crate) fn new(box_offsets: LogicalVec2<AbsoluteBoxOffsets>) -> Self {
        HoistedSharedFragment {
            fragment: None,
            box_offsets,
        }
    }
}

impl HoistedSharedFragment {
    /// In some cases `inset: auto`-positioned elements do not know their precise
    /// position until after they're hoisted. This lets us adjust auto values
    /// after the fact.
    pub(crate) fn adjust_offsets(&mut self, offsets: LogicalVec2<Au>) {
        self.box_offsets.inline.adjust_offset(offsets.inline);
        self.box_offsets.block.adjust_offset(offsets.block);
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) enum AbsoluteBoxOffsets {
    StaticStart {
        start: Au,
    },
    Start {
        start: LengthPercentage,
    },
    End {
        end: LengthPercentage,
    },
    Both {
        start: LengthPercentage,
        end: LengthPercentage,
    },
}

impl AbsoluteBoxOffsets {
    pub(crate) fn both_specified(&self) -> bool {
        matches!(self, AbsoluteBoxOffsets::Both { .. })
    }

    pub(crate) fn adjust_offset(&mut self, new_offset: Au) {
        if let AbsoluteBoxOffsets::StaticStart { ref mut start } = *self {
            *start = new_offset
        }
    }
}
