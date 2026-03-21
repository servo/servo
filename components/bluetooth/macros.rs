/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[cfg(feature = "bluetooth-test")]
macro_rules! get_inner_and_call_test_func {
    ($enum_value: expr, $enum_type: ident, $function_name: ident, $value: expr) => {
        match $enum_value {
            &$enum_type::Mock(ref fake) => fake.$function_name($value),
            #[cfg(feature = "native-bluetooth")]
            _ => Err(Box::from(
                "Error! Test functions are not supported on real devices!",
            )),
        }
    };

    ($enum_value: expr, $enum_type: ident, $function_name: ident) => {
        match $enum_value {
            &$enum_type::Mock(ref fake) => fake.$function_name(),
            #[cfg(feature = "native-bluetooth")]
            _ => Err(Box::from(
                "Error! Test functions are not supported on real devices!",
            )),
        }
    };
}

#[cfg(feature = "bluetooth-test")]
pub(crate) use get_inner_and_call_test_func;
