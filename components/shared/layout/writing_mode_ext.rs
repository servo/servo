/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use style::logical_geometry::WritingMode;

pub trait WritingModeExt {
    fn is_inline_flipped(&self) -> bool;
    fn is_block_flipped(&self) -> bool;
}

impl WritingModeExt for WritingMode {
    fn is_inline_flipped(&self) -> bool {
        !self.is_inline_tb()
    }
    fn is_block_flipped(&self) -> bool {
        self.is_vertical() && !self.is_vertical_lr()
    }
}
