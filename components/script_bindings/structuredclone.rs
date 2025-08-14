/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/// A marker to ensure that the `[Serializable]` attribute is present on
/// types that can be serialized. This trait should not be implemented manually.
pub trait MarkedAsSerializableInIdl {
    /// Used to define compile-time assertions about the type implementing this trait.
    fn assert_serializable();
}

/// A marker to ensure that the `[Transferable]` attribute is present on
/// types that can be transferred. This trait should not be implemented manually.
pub trait MarkedAsTransferableInIdl {
    /// Used to define compile-time assertions about the type implementing this trait.
    fn assert_transferable();
}
