/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Values for CSS Box Alignment properties
//!
//! https://drafts.csswg.org/css-align/

use values::computed::{Context, ToComputedValue};
use values::specified;

pub use super::specified::{AlignContent, JustifyContent, AlignItems, SelfAlignment};
pub use super::specified::{AlignSelf, JustifySelf};

/// The computed value for the `justify-items` property.
///
/// Need to carry around both the specified and computed value to handle the
/// special legacy keyword without destroying style sharing.
///
/// In particular, `justify-items` is a reset property, so we ought to be able
/// to share its computed representation across elements as long as they match
/// the same rules. Except that it's not true if the specified value for
/// `justify-items` is `auto` and the computed value of the parent has the
/// `legacy` modifier.
///
/// So instead of computing `auto` "normally" looking at get_parent_position(),
/// marking it as uncacheable, we carry the specified value around and handle
/// the special case in `StyleAdjuster` instead, only when the result of the
/// computation would vary.
///
/// Note that we also need to special-case this property in matching.rs, in
/// order to properly handle changes to the legacy keyword... This all kinda
/// sucks :(.
///
/// See the discussion in https://bugzil.la/1384542.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ToCss)]
pub struct JustifyItems {
    /// The specified value for the property. Can contain `auto`.
    #[css(skip)]
    pub specified: specified::JustifyItems,
    /// The computed value for the property. Cannot contain `auto`.
    pub computed: specified::JustifyItems,
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
