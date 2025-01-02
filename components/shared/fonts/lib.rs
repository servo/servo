/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

use std::sync::Arc;

use malloc_size_of_derive::MallocSizeOf;
use range::{int_range_index, RangeIndex};
use serde::{Deserialize, Serialize};

int_range_index! {
    #[derive(Deserialize, MallocSizeOf, Serialize)]
    /// An index that refers to a byte offset in a text run. This could
    /// the middle of a glyph.
    struct ByteIndex(isize)
}

pub type WebFontLoadFinishedCallback = Arc<dyn Fn(bool) + Send + Sync + 'static>;
