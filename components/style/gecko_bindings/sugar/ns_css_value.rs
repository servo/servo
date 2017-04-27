/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Little helpers for `nsCSSValue`.

use app_units::Au;
use gecko_bindings::bindings;
use gecko_bindings::structs::{nsCSSValue, nsCSSUnit};
use gecko_bindings::structs::{nsCSSValue_Array, nscolor};
use gecko_string_cache::Atom;
use std::mem;
use std::ops::{Index, IndexMut};
use std::slice;
use values::computed::LengthOrPercentage;
use values::specified::url::SpecifiedUrl;

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

    /// Checks if it is an integer and returns it if so
    pub fn integer(&self) -> Option<i32> {
        if self.mUnit == nsCSSUnit::eCSSUnit_Integer ||
           self.mUnit == nsCSSUnit::eCSSUnit_Enumerated ||
           self.mUnit == nsCSSUnit::eCSSUnit_EnumColor {
            Some(unsafe { *self.mValue.mInt.as_ref() })
        } else {
            None
        }
    }

    /// Checks if it is an RGBA color, returning it if so
    /// Only use it with colors set by SetColorValue(),
    /// which always sets RGBA colors
    pub fn color_value(&self) -> Option<nscolor> {
        if self.mUnit == nsCSSUnit::eCSSUnit_RGBAColor {
            Some(unsafe { *self.mValue.mColor.as_ref() })
        } else {
            None
        }
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

    /// Sets LengthOrPercentage value to this nsCSSValue.
    pub unsafe fn set_lop(&mut self, lop: LengthOrPercentage) {
        match lop {
            LengthOrPercentage::Length(au) => {
                bindings::Gecko_CSSValue_SetAbsoluteLength(self, au.0)
            }
            LengthOrPercentage::Percentage(pc) => {
                bindings::Gecko_CSSValue_SetPercentage(self, pc)
            }
            LengthOrPercentage::Calc(calc) => {
                bindings::Gecko_CSSValue_SetCalc(self, calc.into())
            }
        }
    }

    /// Returns LengthOrPercentage value.
    pub unsafe fn get_lop(&self) -> LengthOrPercentage {
        match self.mUnit {
            nsCSSUnit::eCSSUnit_Pixel => {
                LengthOrPercentage::Length(Au(bindings::Gecko_CSSValue_GetAbsoluteLength(self)))
            },
            nsCSSUnit::eCSSUnit_Percent => {
                LengthOrPercentage::Percentage(bindings::Gecko_CSSValue_GetPercentage(self))
            },
            nsCSSUnit::eCSSUnit_Calc => {
                LengthOrPercentage::Calc(bindings::Gecko_CSSValue_GetCalc(self).into())
            },
            x => panic!("The unit should not be {:?}", x),
        }
    }

    /// Set to a normal value
    pub fn set_normal(&mut self) {
        unsafe { bindings::Gecko_CSSValue_SetNormal(self) }
    }

    fn set_string_internal(&mut self, s: &str, unit: nsCSSUnit) {
        unsafe { bindings::Gecko_CSSValue_SetString(self, s.as_ptr(), s.len() as u32, unit) }
    }

    fn set_string_from_atom_internal(&mut self, s: &Atom, unit: nsCSSUnit) {
        unsafe { bindings::Gecko_CSSValue_SetStringFromAtom(self, s.as_ptr(), unit) }
    }

    /// Set to a string value
    pub fn set_string(&mut self, s: &str) {
        self.set_string_internal(s, nsCSSUnit::eCSSUnit_String)
    }

    /// Set to a string value from the given atom
    pub fn set_string_from_atom(&mut self, s: &Atom) {
        self.set_string_from_atom_internal(s, nsCSSUnit::eCSSUnit_String)
    }

    /// Set to a ident value from the given atom
    pub fn set_ident_from_atom(&mut self, s: &Atom) {
        self.set_string_from_atom_internal(s, nsCSSUnit::eCSSUnit_Ident)
    }

    /// Set to an identifier value
    pub fn set_ident(&mut self, s: &str) {
        self.set_string_internal(s, nsCSSUnit::eCSSUnit_Ident)
    }

    /// Set to a font format
    pub fn set_font_format(&mut self, s: &str) {
        self.set_string_internal(s, nsCSSUnit::eCSSUnit_Font_Format);
    }

    /// Set to a local font value
    pub fn set_local_font(&mut self, s: &Atom) {
        self.set_string_from_atom_internal(s, nsCSSUnit::eCSSUnit_Local_Font);
    }

    fn set_int_internal(&mut self, value: i32, unit: nsCSSUnit) {
        unsafe { bindings::Gecko_CSSValue_SetInt(self, value, unit) }
    }

    /// Set to an integer value
    pub fn set_integer(&mut self, value: i32) {
        self.set_int_internal(value, nsCSSUnit::eCSSUnit_Integer)
    }

    /// Set to an enumerated value
    pub fn set_enum<T: Into<i32>>(&mut self, value: T) {
        self.set_int_internal(value.into(), nsCSSUnit::eCSSUnit_Enumerated);
    }

    /// Set to a url value
    pub fn set_url(&mut self, url: &SpecifiedUrl) {
        unsafe { bindings::Gecko_CSSValue_SetURL(self, url.for_ffi()) }
    }

    /// Set to an array of given length
    pub fn set_array(&mut self, len: i32) -> &mut nsCSSValue_Array {
        unsafe { bindings::Gecko_CSSValue_SetArray(self, len) }
        unsafe { self.mValue.mArray.as_mut().as_mut() }.unwrap()
    }

    /// Generic set from any value that implements the ToNsCssValue trait.
    pub fn set_from<T: ToNsCssValue>(&mut self, value: &T) {
        value.convert(self)
    }
}

impl Drop for nsCSSValue {
    fn drop(&mut self) {
        unsafe { bindings::Gecko_CSSValue_Drop(self) };
    }
}

impl nsCSSValue_Array {
    /// Return the length of this `nsCSSValue::Array`
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

    /// Get the array as a mutable slice of nsCSSValues.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [nsCSSValue] {
        unsafe { slice::from_raw_parts_mut(self.buffer() as *mut _, self.len()) }
    }
}

impl Index<usize> for nsCSSValue_Array {
    type Output = nsCSSValue;
    #[inline]
    fn index(&self, i: usize) -> &nsCSSValue {
        &self.as_slice()[i]
    }
}

impl IndexMut<usize> for nsCSSValue_Array {
    #[inline]
    fn index_mut(&mut self, i: usize) -> &mut nsCSSValue {
        &mut self.as_mut_slice()[i]
    }
}

/// Generic conversion to nsCSSValue
pub trait ToNsCssValue {
    /// Convert
    fn convert(&self, nscssvalue: &mut nsCSSValue);
}
