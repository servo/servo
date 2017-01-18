/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Little helpers for `nsCSSValue`.

use gecko_bindings::bindings::Gecko_CSSValue_Drop;
use gecko_bindings::structs::{nsCSSValue, nsCSSUnit, nsCSSValue_Array};
use std::mem;
use std::ops::Index;
use std::slice;

impl nsCSSValue {
    /// Create a CSSValue with null unit, useful to be used as a return value.
    #[inline]
    pub fn null() -> Self {
        unsafe { mem::zeroed() }
    }

    /// Returns this nsCSSValue value as an integer, unchecked in release
    /// builds.
    pub fn integer_unchecked(&self) -> i32 {
        debug_assert!(self.mUnit == nsCSSUnit::eCSSUnit_Integer ||
                      self.mUnit == nsCSSUnit::eCSSUnit_Enumerated ||
                      self.mUnit == nsCSSUnit::eCSSUnit_EnumColor);
        unsafe { *self.mValue.mInt.as_ref() }
    }

    /// Returns this nsCSSValue value as a floating point value, unchecked in
    /// release builds.
    pub fn float_unchecked(&self) -> f32 {
        debug_assert!(nsCSSUnit::eCSSUnit_Number as u32 <= self.mUnit as u32);
        unsafe { *self.mValue.mFloat.as_ref() }
    }

    /// Returns this nsCSSValue as a nsCSSValue::Array, unchecked in release
    /// builds.
    pub unsafe fn array_unchecked(&self) -> &nsCSSValue_Array {
        debug_assert!(nsCSSUnit::eCSSUnit_Array as u32 <= self.mUnit as u32 &&
                      self.mUnit as u32 <= nsCSSUnit::eCSSUnit_Calc_Divided as u32);
        let array = *self.mValue.mArray.as_ref();
        debug_assert!(!array.is_null());
        &*array
    }
}

impl Drop for nsCSSValue {
    fn drop(&mut self) {
        unsafe { Gecko_CSSValue_Drop(self) };
    }
}

impl nsCSSValue_Array {
    /// Return the length of this `nsCSSShadowArray`
    #[inline]
    pub fn len(&self) -> usize {
        self.mCount
    }

    #[inline]
    fn buffer(&self) -> *const nsCSSValue {
        self.mArray.as_ptr()
    }

    /// Get the array as a slice of nsCSSValues.
    #[inline]
    pub fn as_slice(&self) -> &[nsCSSValue] {
        unsafe { slice::from_raw_parts(self.buffer(), self.len()) }
    }
}

impl Index<usize> for nsCSSValue_Array {
    type Output = nsCSSValue;
    #[inline]
    fn index(&self, i: usize) -> &nsCSSValue {
        &self.as_slice()[i]
    }
}
