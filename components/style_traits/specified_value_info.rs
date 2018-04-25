/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Value information for devtools.

use servo_arc::Arc;
use std::ops::Range;

/// Type of value that a property supports. This is used by Gecko's
/// devtools to make sense about value it parses, and types listed
/// here should match TYPE_* constants in InspectorUtils.webidl.
///
/// XXX This should really be a bitflags rather than a namespace mod,
/// but currently we cannot use bitflags in const.
#[allow(non_snake_case)]
pub mod CssType {
    /// <color>
    pub const COLOR: u8 = 1 << 0;
    /// <gradient>
    pub const GRADIENT: u8 = 1 << 1;
    /// <timing-function>
    pub const TIMING_FUNCTION: u8 = 1 << 2;
}

/// Information of values of a given specified value type.
pub trait SpecifiedValueInfo {
    /// Supported CssTypes by the given value type.
    ///
    /// XXX This should be typed CssType when that becomes a bitflags.
    /// Currently we cannot do so since bitflags cannot be used in constant.
    const SUPPORTED_TYPES: u8 = 0;
}

impl SpecifiedValueInfo for bool {}
impl SpecifiedValueInfo for f32 {}
impl SpecifiedValueInfo for i8 {}
impl SpecifiedValueInfo for i32 {}
impl SpecifiedValueInfo for u8 {}
impl SpecifiedValueInfo for u16 {}
impl SpecifiedValueInfo for u32 {}
impl SpecifiedValueInfo for str {}
impl SpecifiedValueInfo for String {}

impl<T: SpecifiedValueInfo + ?Sized> SpecifiedValueInfo for Box<T> {
    const SUPPORTED_TYPES: u8 = T::SUPPORTED_TYPES;
}

impl<T: SpecifiedValueInfo> SpecifiedValueInfo for [T] {
    const SUPPORTED_TYPES: u8 = T::SUPPORTED_TYPES;
}

macro_rules! impl_generic_specified_value_info {
    ($ty:ident<$param:ident>) => {
        impl<$param: SpecifiedValueInfo> SpecifiedValueInfo for $ty<$param> {
            const SUPPORTED_TYPES: u8 = $param::SUPPORTED_TYPES;
        }
    }
}
impl_generic_specified_value_info!(Option<T>);
impl_generic_specified_value_info!(Vec<T>);
impl_generic_specified_value_info!(Arc<T>);
impl_generic_specified_value_info!(Range<Idx>);

impl<T1, T2> SpecifiedValueInfo for (T1, T2)
where
    T1: SpecifiedValueInfo,
    T2: SpecifiedValueInfo,
{
    const SUPPORTED_TYPES: u8 = T1::SUPPORTED_TYPES | T2::SUPPORTED_TYPES;
}
