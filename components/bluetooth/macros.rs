/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

macro_rules! get_inner_and_call(
    ($enum_value: expr, $enum_type: ident, $function_name: ident) => {
        match $enum_value {
            #[cfg(all(target_os = "linux", feature = "native-bluetooth"))]
            &$enum_type::Bluez(ref bluez) => bluez.$function_name(),
            #[cfg(all(target_os = "android", feature = "native-bluetooth"))]
            &$enum_type::Android(ref android) => android.$function_name(),
            #[cfg(all(target_os = "macos", feature = "native-bluetooth"))]
            &$enum_type::Mac(ref mac) => mac.$function_name(),
            #[cfg(not(any(all(target_os = "linux", feature = "native-bluetooth"),
                          all(target_os = "android", feature = "native-bluetooth"),
                          all(target_os = "macos", feature = "native-bluetooth"))))]
            &$enum_type::Empty(ref empty) => empty.$function_name(),
            #[cfg(feature = "bluetooth-test")]
            &$enum_type::Mock(ref fake) => fake.$function_name(),
        }
    };

    (@with_bluez_offset, $enum_value: expr, $enum_type: ident, $function_name: ident) => {
        match $enum_value {
            #[cfg(all(target_os = "linux", feature = "native-bluetooth"))]
            &$enum_type::Bluez(ref bluez) => bluez.$function_name(None),
            #[cfg(all(target_os = "android", feature = "native-bluetooth"))]
            &$enum_type::Android(ref android) => android.$function_name(),
            #[cfg(all(target_os = "macos", feature = "native-bluetooth"))]
            &$enum_type::Mac(ref mac) => mac.$function_name(),
            #[cfg(not(any(all(target_os = "linux", feature = "native-bluetooth"),
                          all(target_os = "android", feature = "native-bluetooth"),
                          all(target_os = "macos", feature = "native-bluetooth"))))]
            &$enum_type::Empty(ref empty) => empty.$function_name(),
            #[cfg(feature = "bluetooth-test")]
            &$enum_type::Mock(ref fake) => fake.$function_name(),
        }
    };

    ($enum_value: expr, $enum_type: ident, $function_name: ident, $value: expr) => {
        match $enum_value {
            #[cfg(all(target_os = "linux", feature = "native-bluetooth"))]
            &$enum_type::Bluez(ref bluez) => bluez.$function_name($value),
            #[cfg(all(target_os = "android", feature = "native-bluetooth"))]
            &$enum_type::Android(ref android) => android.$function_name($value),
            #[cfg(all(target_os = "macos", feature = "native-bluetooth"))]
            &$enum_type::Mac(ref mac) => mac.$function_name($value),
            #[cfg(not(any(all(target_os = "linux", feature = "native-bluetooth"),
                          all(target_os = "android", feature = "native-bluetooth"),
                          all(target_os = "macos", feature = "native-bluetooth"))))]
            &$enum_type::Empty(ref empty) => empty.$function_name($value),
            #[cfg(feature = "bluetooth-test")]
            &$enum_type::Mock(ref fake) => fake.$function_name($value),
        }
    };

    (@with_bluez_offset, $enum_value: expr, $enum_type: ident, $function_name: ident, $value: expr) => {
        match $enum_value {
            #[cfg(all(target_os = "linux", feature = "native-bluetooth"))]
            &$enum_type::Bluez(ref bluez) => bluez.$function_name($value, None),
            #[cfg(all(target_os = "android", feature = "native-bluetooth"))]
            &$enum_type::Android(ref android) => android.$function_name($value),
            #[cfg(all(target_os = "macos", feature = "native-bluetooth"))]
            &$enum_type::Mac(ref mac) => mac.$function_name($value),
            #[cfg(not(any(all(target_os = "linux", feature = "native-bluetooth"),
                          all(target_os = "android", feature = "native-bluetooth"),
                          all(target_os = "macos", feature = "native-bluetooth"))))]
            &$enum_type::Empty(ref empty) => empty.$function_name($value),
            #[cfg(feature = "bluetooth-test")]
            &$enum_type::Mock(ref fake) => fake.$function_name($value),
        }
    };
);

#[cfg(feature = "bluetooth-test")]
macro_rules! get_inner_and_call_test_func {
    ($enum_value: expr, $enum_type: ident, $function_name: ident, $value: expr) => {
        match $enum_value {
            &$enum_type::Mock(ref fake) => fake.$function_name($value),
            _ => Err(Box::from(
                "Error! Test functions are not supported on real devices!",
            )),
        }
    };

    ($enum_value: expr, $enum_type: ident, $function_name: ident) => {
        match $enum_value {
            &$enum_type::Mock(ref fake) => fake.$function_name(),
            _ => Err(Box::from(
                "Error! Test functions are not supported on real devices!",
            )),
        }
    };
}

pub(crate) use get_inner_and_call;
#[cfg(feature = "bluetooth-test")]
pub(crate) use get_inner_and_call_test_func;
