/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! `list` computed values.

#[cfg(feature = "gecko")]
pub use crate::values::specified::list::ListStyleType;
pub use crate::values::specified::list::{QuotePair, Quotes};

use servo_arc::Arc;

lazy_static! {
    static ref INITIAL_QUOTES: Arc<Box<[QuotePair]>> = Arc::new(
        vec![
            QuotePair {
                opening: "\u{201c}".to_owned().into_boxed_str(),
                closing: "\u{201d}".to_owned().into_boxed_str(),
            },
            QuotePair {
                opening: "\u{2018}".to_owned().into_boxed_str(),
                closing: "\u{2019}".to_owned().into_boxed_str(),
            },
        ]
        .into_boxed_slice()
    );
}

impl Quotes {
    /// Initial value for `quotes`.
    #[inline]
    pub fn get_initial_value() -> Quotes {
        Quotes(INITIAL_QUOTES.clone())
    }
}
