/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Value information for devtools.

use servo_arc::Arc;
use std::ops::Range;
use std::sync::Arc as StdArc;

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

/// See SpecifiedValueInfo::collect_completion_keywords.
pub type KeywordsCollectFn<'a> = &'a mut FnMut(&[&'static str]);

/// Information of values of a given specified value type.
///
/// This trait is derivable with `#[derive(SpecifiedValueInfo)]`.
///
/// The algorithm traverses the type definition. For `SUPPORTED_TYPES`,
/// it puts an or'ed value of `SUPPORTED_TYPES` of all types it finds.
/// For `collect_completion_keywords`, it recursively invokes this
/// method on types found, and lists all keyword values and function
/// names following the same rule as `ToCss` in that method.
///
/// Some attributes of `ToCss` can affect the behavior, specifically:
/// * If `#[css(function)]` is found, the content inside the annotated
///   variant (or the whole type) isn't traversed, only the function
///   name is listed in `collect_completion_keywords`.
/// * If `#[css(skip)]` is found, the content inside the variant or
///   field is ignored.
/// * Values listed in `#[css(if_empty)]`, `#[parse(aliases)]`, and
///   `#[css(keyword)]` are added into `collect_completion_keywords`.
///
/// In addition to `css` attributes, it also has `value_info` helper
/// attributes, including:
/// * `#[value_info(ty = "TYPE")]` can be used to specify a constant
///   from `CssType` to `SUPPORTED_TYPES`.
/// * `#[value_info(other_values = "value1,value2")]` can be used to
///   add other values related to a field, variant, or the type itself
///   into `collect_completion_keywords`.
/// * `#[value_info(starts_with_keyword)]` can be used on variants to
///   add the name of a non-unit variant (serialized like `ToCss`) into
///   `collect_completion_keywords`.
pub trait SpecifiedValueInfo {
    /// Supported CssTypes by the given value type.
    ///
    /// XXX This should be typed CssType when that becomes a bitflags.
    /// Currently we cannot do so since bitflags cannot be used in constant.
    const SUPPORTED_TYPES: u8 = 0;

    /// Collect value starting words for the given specified value type.
    /// This includes keyword and function names which can appear at the
    /// beginning of a value of this type.
    ///
    /// Caller should pass in a callback function to accept the list of
    /// values. The callback function can be called multiple times, and
    /// some values passed to the callback may be duplicate.
    fn collect_completion_keywords(_f: KeywordsCollectFn) {}
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

#[cfg(feature = "servo")]
impl SpecifiedValueInfo for ::servo_atoms::Atom {}
#[cfg(feature = "servo")]
impl SpecifiedValueInfo for ::servo_url::ServoUrl {}

impl<T: SpecifiedValueInfo + ?Sized> SpecifiedValueInfo for Box<T> {
    const SUPPORTED_TYPES: u8 = T::SUPPORTED_TYPES;
    fn collect_completion_keywords(f: KeywordsCollectFn) {
        T::collect_completion_keywords(f);
    }
}

impl<T: SpecifiedValueInfo> SpecifiedValueInfo for [T] {
    const SUPPORTED_TYPES: u8 = T::SUPPORTED_TYPES;
    fn collect_completion_keywords(f: KeywordsCollectFn) {
        T::collect_completion_keywords(f);
    }
}

macro_rules! impl_generic_specified_value_info {
    ($ty:ident<$param:ident>) => {
        impl<$param: SpecifiedValueInfo> SpecifiedValueInfo for $ty<$param> {
            const SUPPORTED_TYPES: u8 = $param::SUPPORTED_TYPES;
            fn collect_completion_keywords(f: KeywordsCollectFn) {
                $param::collect_completion_keywords(f);
            }
        }
    }
}
impl_generic_specified_value_info!(Option<T>);
impl_generic_specified_value_info!(Vec<T>);
impl_generic_specified_value_info!(Arc<T>);
impl_generic_specified_value_info!(StdArc<T>);
impl_generic_specified_value_info!(Range<Idx>);

impl<T1, T2> SpecifiedValueInfo for (T1, T2)
where
    T1: SpecifiedValueInfo,
    T2: SpecifiedValueInfo,
{
    const SUPPORTED_TYPES: u8 = T1::SUPPORTED_TYPES | T2::SUPPORTED_TYPES;

    fn collect_completion_keywords(f: KeywordsCollectFn) {
        T1::collect_completion_keywords(f);
        T2::collect_completion_keywords(f);
    }
}
