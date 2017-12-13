/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! `list` computed values.

pub use values::specified::list::Quotes;

impl Quotes {
    /// Initial value for `quotes`
    #[inline]
    pub fn get_initial_value() -> Quotes {
        Quotes(vec![
            ("\u{201c}".to_owned(), "\u{201d}".to_owned()),
            ("\u{2018}".to_owned(), "\u{2019}".to_owned()),
        ])
    }
}
