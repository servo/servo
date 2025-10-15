/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/// A version of the `Into<T>` trait from the standard library that can be used
/// to convert between two types that are not defined in the script crate.
/// This is intended to be used on dict/enum types generated from WebIDL once
/// those types are moved out of the script crate.
pub(crate) trait Convert<T> {
    fn convert(self) -> T;
}

/// A version of the `TryInto<T>` trait from the standard library that can be used
/// to convert between two types that are not defined in the script crate.
/// This is intended to be used on dict/enum types generated from WebIDL once
/// those types are moved out of the script crate.
#[cfg(feature = "webgpu")]
pub(crate) trait TryConvert<T> {
    type Error;

    fn try_convert(self) -> Result<T, Self::Error>;
}
