/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! `list` computed values.

pub use values::specified::list::{ListStyleImage, Quotes};

impl Quotes {
    /// Initial value for `quotes`.
    ///
    /// FIXME(emilio): This should ideally not allocate.
    #[inline]
    pub fn get_initial_value() -> Quotes {
        Quotes(vec![
            (
                "\u{201c}".to_owned().into_boxed_str(),
                "\u{201d}".to_owned().into_boxed_str(),
            ),
            (
                "\u{2018}".to_owned().into_boxed_str(),
                "\u{2019}".to_owned().into_boxed_str(),
            ),
        ].into_boxed_slice())
    }
}
