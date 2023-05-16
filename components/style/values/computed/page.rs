/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Computed @page at-rule properties

use crate::values::computed::length::NonNegativeLength;
use crate::values::generics;
use crate::values::generics::size::Size2D;

pub use generics::page::Orientation;
pub use generics::page::PaperSize;
/// Computed value of the @page size descriptor
pub type PageSize = generics::page::GenericPageSize<Size2D<NonNegativeLength>>;
