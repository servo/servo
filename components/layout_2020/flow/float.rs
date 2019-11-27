/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::formatting_contexts::IndependentFormattingContext;
use servo_arc::Arc;
use style::properties::ComputedValues;

#[derive(Debug)]
pub(crate) struct FloatBox {
    pub contents: IndependentFormattingContext,
}

/// Data kept during layout about the floats in a given block formatting context.
pub(crate) struct FloatContext {
    // TODO
}

impl FloatContext {
    pub fn new() -> Self {
        FloatContext {}
    }
}
