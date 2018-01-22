/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Values for CSS Box Alignment properties
//!
//! https://drafts.csswg.org/css-align/

use std::fmt;
use style_traits::{CssWriter, ToCss};
use values::computed::{Context, ToComputedValue};
use values::specified;

pub use super::specified::{AlignItems, AlignJustifyContent, AlignJustifySelf};

/// The computed value for the `justify-items` property.
///
/// Need to carry around both the specified and computed value to handle the
/// special legacy keyword. Sigh.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JustifyItems {
    /// The specified value for the property. Can contain `auto`.
    pub specified: specified::JustifyItems,
    /// The computed value for the property. Cannot contain `auto`.
    pub computed: specified::JustifyItems,
}

impl ToCss for JustifyItems {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
        where W: fmt::Write,
    {
        self.computed.to_css(dest)
    }
}

impl JustifyItems {
    /// Returns the `auto` value.
    pub fn auto() -> Self {
        Self {
            specified: specified::JustifyItems::auto(),
            computed: specified::JustifyItems::normal(),
        }
    }
}

impl ToComputedValue for specified::JustifyItems {
    type ComputedValue = JustifyItems;

    /// <https://drafts.csswg.org/css-align/#valdef-justify-items-legacy>
    fn to_computed_value(&self, _context: &Context) -> JustifyItems {
        use values::specified::align;
        let specified = *self;
        let computed =
            if self.0 != align::AlignFlags::AUTO {
                *self
            } else {
                // If the inherited value of `justify-items` includes the
                // `legacy` keyword, `auto` computes to the inherited value,
                // but we assume it computes to `normal`, and handle that
                // special-case in StyleAdjuster.
                Self::normal()
            };

        JustifyItems { specified, computed }
    }

    #[inline]
    fn from_computed_value(computed: &JustifyItems) -> Self {
        computed.specified
    }
}
