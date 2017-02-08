/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::mem::size_of;
use style::properties::PropertyDeclaration;

#[test]
fn size_of_property_declaration() {
    let old = 240;
    let new = size_of::<PropertyDeclaration>();
    if new < old {
        panic!("Your changes have decreased the stack size of PropertyDeclaration enum from {} to {}. \
                Good work! Please update the size in tests/unit/style/size_of.rs.",
                old, new)
    } else if new > old {
        panic!("Your changes have increased the stack size of PropertyDeclaration enum from {} to {}. \
                These enum is present in large quantities in the style, and increasing the size \
                may dramatically affect our memory footprint. Please consider using `boxed=\"True\"` in \
                the longhand If you feel that the increase is necessary, update to the new size in \
                tests/unit/style/size_of.rs.",
                old, new)
    }
}
