/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Legacy presentational attributes defined in the HTML5 specification: `<td width>`,
//! `<input size>`, and so forth.

/// Legacy presentational attributes that take a length as defined in HTML5 § 2.4.4.4.
pub enum LengthAttribute {
    /// `<td width>`
    WidthLengthAttribute,
}

/// Legacy presentational attributes that take an integer as defined in HTML5 § 2.4.4.2.
pub enum IntegerAttribute {
    /// `<input size>`
    SizeIntegerAttribute,
}

