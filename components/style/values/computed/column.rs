/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Computed types for the column properties.

use crate::values::computed::PositiveInteger;
use crate::values::generics::column::ColumnCount as GenericColumnCount;

/// A computed type for `column-count` values.
pub type ColumnCount = GenericColumnCount<PositiveInteger>;
