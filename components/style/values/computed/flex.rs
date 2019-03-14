/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Computed types for CSS values related to flexbox.

use crate::values::computed::Size;
use crate::values::generics::flex::FlexBasis as GenericFlexBasis;

/// A computed value for the `flex-basis` property.
pub type FlexBasis = GenericFlexBasis<Size>;

impl FlexBasis {
    /// `auto`
    #[inline]
    pub fn auto() -> Self {
        GenericFlexBasis::Size(Size::auto())
    }
}
