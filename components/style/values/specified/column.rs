/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified types for the column properties.

use crate::values::generics::column::ColumnCount as GenericColumnCount;
use crate::values::specified::PositiveInteger;

/// A specified type for `column-count` values.
pub type ColumnCount = GenericColumnCount<PositiveInteger>;
