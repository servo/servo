/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// The "real" public API
pub use self::selector_matching::{Stylist, StylesheetOrigin};


// Things that need to be public to make the compiler happy
pub mod stylesheets;
pub mod errors;
pub mod selectors;
pub mod selector_matching;
pub mod properties;
pub mod namespaces;
pub mod media_queries;
pub mod parsing_utils;

#[cfg(test)]
mod tests;
