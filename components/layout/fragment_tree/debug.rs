/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;

use app_units::Au;

use crate::geom::{PhysicalRect, PhysicalSides};

pub(crate) struct PrintableRect(pub PhysicalRect<Au>);

impl fmt::Debug for PrintableRect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}px,{}px {}pxx{}px]",
            self.0.origin.x.to_f32_px(),
            self.0.origin.y.to_f32_px(),
            self.0.size.width.to_f32_px(),
            self.0.size.height.to_f32_px(),
        )
    }
}

pub(crate) struct PrintableSides(pub PhysicalSides<Au>);

impl fmt::Debug for PrintableSides {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_zero() {
            return Ok(());
        }
        if self.0.left == self.0.right && self.0.left == self.0.top && self.0.left == self.0.bottom
        {
            return write!(f, "[{}px]", self.0.left.to_f32_px());
        }
        write!(
            f,
            "[{}px {}px {}px {}px]",
            self.0.top.to_f32_px(),
            self.0.right.to_f32_px(),
            self.0.bottom.to_f32_px(),
            self.0.left.to_f32_px(),
        )
    }
}
