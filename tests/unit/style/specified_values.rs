/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use style;

#[cfg(all(test, target_pointer_width = "64"))]
#[test]
fn size_of_specified_values() {
    use std::mem::size_of;
    let threshold = 24;

    let mut bad_properties = vec![];

    macro_rules! check_property {
        ( $( { $name: ident, $boxed: expr } )+ ) => {
            $(
                let size = size_of::<style::properties::longhands::$name::SpecifiedValue>();
                let is_boxed = $boxed;
                if (!is_boxed && size > threshold) || (is_boxed && size <= threshold) {
                    bad_properties.push((stringify!($name), size, is_boxed));
                }
            )+
        }
    }

    longhand_properties_idents!(check_property);

    let mut failing_messages = vec![];

    for bad_prop in bad_properties {
        if !bad_prop.2 {
            failing_messages.push(
                format!("Your changes have increased the size of {} SpecifiedValue to {}. The threshold is \
                        currently {}. SpecifiedValues affect size of PropertyDeclaration enum and \
                        increasing the size may negative affect style system performance. Please consider \
                        using `boxed=\"True\"` in this longhand.",
                        bad_prop.0, bad_prop.1, threshold));
        } else if bad_prop.2 {
            failing_messages.push(
                format!("Your changes have decreased the size of {} SpecifiedValue to {}. Good work! \
                        The threshold is currently {}. Please consider removing `boxed=\"True\"` from this longhand.",
                        bad_prop.0, bad_prop.1, threshold));
        }
    }

    if !failing_messages.is_empty() {
        panic!("{}", failing_messages.join("\n\n"));
    }
}
