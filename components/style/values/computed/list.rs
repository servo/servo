/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! `list` computed values.

#[cfg(feature = "gecko")]
pub use crate::values::specified::list::ListStyleType;
pub use crate::values::specified::list::MozListReversed;
pub use crate::values::specified::list::Quotes;

impl Quotes {
    /// Initial value for `quotes`.
    #[inline]
    pub fn get_initial_value() -> Quotes {
        Quotes::Auto
    }
}
