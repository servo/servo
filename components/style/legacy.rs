/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Legacy presentational attributes defined in the HTML5 specification: `<td width>`,
//! `<input size>`, and so forth.

use properties::PropertyDeclaration;
use selectors::matching::DeclarationBlock;
use std::sync::Arc;

/// A convenience function to create a declaration block from a single declaration. This is
/// primarily used in `synthesize_rules_for_legacy_attributes`.
#[inline]
pub fn from_declaration(rule: PropertyDeclaration) -> DeclarationBlock<Vec<PropertyDeclaration>> {
    DeclarationBlock::from_declarations(Arc::new(vec![rule]))
}
