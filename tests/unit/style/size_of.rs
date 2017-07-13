/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use style::invalidation::element::invalidation_map::Dependency;
use style::properties;

size_of_test!(test_size_of_dependency, Dependency, 16);

size_of_test!(test_size_of_property_declaration, properties::PropertyDeclaration, 32);

// This is huge, but we allocate it on the stack and then never move it,
// we only pass `&mut SourcePropertyDeclaration` references around.
size_of_test!(test_size_of_parsed_declaration, properties::SourcePropertyDeclaration, 576);

#[test]
fn size_of_specified_values() {
    ::style::properties::test_size_of_specified_values();
}
