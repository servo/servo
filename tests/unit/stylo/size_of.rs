/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[test]
fn size_of_property_declaration() {
    ::style::properties::test_size_of_property_declaration();
}

#[test]
fn size_of_specified_values() {
    ::style::properties::test_size_of_specified_values();
}
