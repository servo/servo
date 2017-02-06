/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use style::properties::specified_value_sizes;

 #[test]
fn size_of_specified_values() {
    let threshold = 40;
    let longhands = specified_value_sizes();

    for specified_value in longhands {
        if specified_value.1 >= threshold && !specified_value.2 {
            panic!("Your changes have increased the size of {} SpecifiedValue to {}. The threshold is \
                    currently {}. SpecifiedValues are affect size of PropertyDeclaration enum and \
                    increasing the size may dramatically affect our memory footprint. Please consider \
                    using `boxed=\"True\"` in this longhand.",
                    specified_value.0, specified_value.1, threshold)
        } else if specified_value.1 < threshold && specified_value.2 {
            panic!("Your changes have decreased the size of {} SpecifiedValue to {}. Good work! \
                    The threshold is currently {}. Please consider removing `boxed=\"True\"` from this longhand.",
                    specified_value.0, specified_value.1, threshold)
        }
    }
}
