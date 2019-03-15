/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Little helpers for `nsCSSValue`.

use crate::gecko_bindings::bindings;
use crate::gecko_bindings::structs;
use crate::gecko_bindings::structs::{nsCSSUnit, nsCSSValue};
use crate::gecko_bindings::structs::{nsCSSValueList, nsCSSValue_Array};
use crate::gecko_string_cache::Atom;
use crate::values::computed::{Angle, Length, LengthPercentage, Percentage};
use std::marker::PhantomData;
use std::mem;
use std::ops::{Index, IndexMut};
use std::slice;

impl nsCSSValue {
    /// Create a CSSValue with null unit, useful to be used as a return value.
    #[inline]
    pub fn null() -> Self {
        unsafe { mem::zeroed() }
    }

    /// Returns true if this nsCSSValue is none.
    #[inline]
    pub fn is_none(&self) -> bool {
        self.mUnit == nsCSSUnit::eCSSUnit_None
    }

    /// Returns this nsCSSValue value as an integer, unchecked in release
    /// builds.
    pub fn integer_unchecked(&self) -> i32 {
        debug_assert!(
            self.mUnit == nsCSSUnit::eCSSUnit_Integer ||
                self.mUnit == nsCSSUnit::eCSSUnit_Enumerated
        );
        unsafe { *self.mValue.mInt.as_ref() }
    }

    /// Checks if it is an integer and returns it if so
    pub fn integer(&self) -> Option<i32> {
        if self.mUnit == nsCSSUnit::eCSSUnit_Integer || self.mUnit == nsCSSUnit::eCSSUnit_Enumerated
        {
            Some(unsafe { *self.mValue.mInt.as_ref() })
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
        debug_assert!(
            nsCSSUnit::eCSSUnit_Array as u32 <= self.mUnit as u32 &&
                self.mUnit as u32 <= nsCSSUnit::eCSSUnit_Calc_Plus as u32
        );
        let array = *self.mValue.mArray.as_ref();
        debug_assert!(!array.is_null());
        &*array
    }

    /// Sets LengthPercentage value to this nsCSSValue.
    pub unsafe fn set_length_percentage(&mut self, lp: LengthPercentage) {
        if lp.was_calc {
            return bindings::Gecko_CSSValue_SetCalc(self, lp.into())
        }
        debug_assert!(lp.percentage.is_none() || lp.unclamped_length() == Length::zero());
        if let Some(p) = lp.percentage {
            return self.set_percentage(p.0);
        }
        self.set_px(lp.unclamped_length().px());
    }

    /// Sets a px value to this nsCSSValue.
    pub unsafe fn set_px(&mut self, px: f32) {
        bindings::Gecko_CSSValue_SetPixelLength(self, px)
    }

    /// Sets a percentage value to this nsCSSValue.
    pub unsafe fn set_percentage(&mut self, unit_value: f32) {
        bindings::Gecko_CSSValue_SetPercentage(self, unit_value)
    }

    /// Returns LengthPercentage value.
    pub unsafe fn get_length_percentage(&self) -> LengthPercentage {
        match self.mUnit {
            nsCSSUnit::eCSSUnit_Pixel => {
                LengthPercentage::new(
                    Length::new(bindings::Gecko_CSSValue_GetNumber(self)),
                    None,
                )
            },
            nsCSSUnit::eCSSUnit_Percent => LengthPercentage::new_percent(Percentage(
                bindings::Gecko_CSSValue_GetPercentage(self),
            )),
            nsCSSUnit::eCSSUnit_Calc => {
                bindings::Gecko_CSSValue_GetCalc(self).into()
            },
            _ => panic!("Unexpected unit"),
        }
    }

    /// Returns Length  value.
    pub unsafe fn get_length(&self) -> Length {
        match self.mUnit {
            nsCSSUnit::eCSSUnit_Pixel => Length::new(bindings::Gecko_CSSValue_GetNumber(self)),
            _ => panic!("Unexpected unit"),
        }
    }

    fn set_valueless_unit(&mut self, unit: nsCSSUnit) {
        debug_assert_eq!(self.mUnit, nsCSSUnit::eCSSUnit_Null);
        debug_assert!(
            unit as u32 <= nsCSSUnit::eCSSUnit_DummyInherit as u32,
            "Not a valueless unit"
        );
        self.mUnit = unit;
    }

    /// Set to an auto value
    ///
    /// This method requires the current value to be null.
    pub fn set_auto(&mut self) {
        self.set_valueless_unit(nsCSSUnit::eCSSUnit_Auto);
    }

    /// Set to a normal value
    ///
    /// This method requires the current value to be null.
    pub fn set_normal(&mut self) {
        self.set_valueless_unit(nsCSSUnit::eCSSUnit_Normal);
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

    /// Set to an atom identifier value
    pub fn set_atom_ident(&mut self, s: Atom) {
        unsafe { bindings::Gecko_CSSValue_SetAtomIdent(self, s.into_addrefed()) }
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

    /// Set to a number value
    pub fn set_number(&mut self, number: f32) {
        unsafe { bindings::Gecko_CSSValue_SetFloat(self, number, nsCSSUnit::eCSSUnit_Number) }
    }

    /// Set to an array of given length
    pub fn set_array(&mut self, len: i32) -> &mut nsCSSValue_Array {
        unsafe { bindings::Gecko_CSSValue_SetArray(self, len) }
        unsafe { self.mValue.mArray.as_mut().as_mut() }.unwrap()
    }

    /// Generic set from any value that implements the ToNsCssValue trait.
    pub fn set_from<T: ToNsCssValue>(&mut self, value: T) {
        value.convert(self)
    }

    /// Returns an `Angle` value from this `nsCSSValue`.
    ///
    /// Panics if the unit is not `eCSSUnit_Degree`.
    #[inline]
    pub fn get_angle(&self) -> Angle {
        debug_assert_eq!(self.mUnit, nsCSSUnit::eCSSUnit_Degree);
        Angle::from_degrees(self.float_unchecked())
    }

    /// Sets Angle value to this nsCSSValue.
    pub fn set_angle(&mut self, angle: Angle) {
        debug_assert_eq!(self.mUnit, nsCSSUnit::eCSSUnit_Null);
        self.mUnit = nsCSSUnit::eCSSUnit_Degree;
        unsafe {
            *self.mValue.mFloat.as_mut() = angle.degrees();
        }
    }

    /// Set to a pair value
    ///
    /// This is only supported on the main thread.
    pub fn set_pair(&mut self, x: &nsCSSValue, y: &nsCSSValue) {
        unsafe { bindings::Gecko_CSSValue_SetPair(self, x, y) }
    }

    /// Set to a list value
    ///
    /// This is only supported on the main thread.
    pub fn set_list<I>(&mut self, values: I)
    where
        I: ExactSizeIterator<Item = nsCSSValue>,
    {
        debug_assert!(values.len() > 0, "Empty list is not supported");
        unsafe {
            bindings::Gecko_CSSValue_SetList(self, values.len() as u32);
        }
        debug_assert_eq!(self.mUnit, nsCSSUnit::eCSSUnit_List);
        let list: &mut structs::nsCSSValueList = &mut unsafe {
            self.mValue
                .mList
                .as_ref() // &*nsCSSValueList_heap
                .as_mut()
                .expect("List pointer should be non-null")
        }
        ._base;
        for (item, new_value) in list.into_iter().zip(values) {
            *item = new_value;
        }
    }

    /// Set to a pair list value
    ///
    /// This is only supported on the main thread.
    pub fn set_pair_list<I>(&mut self, mut values: I)
    where
        I: ExactSizeIterator<Item = (nsCSSValue, nsCSSValue)>,
    {
        debug_assert!(values.len() > 0, "Empty list is not supported");
        unsafe {
            bindings::Gecko_CSSValue_SetPairList(self, values.len() as u32);
        }
        debug_assert_eq!(self.mUnit, nsCSSUnit::eCSSUnit_PairList);
        let mut item_ptr = &mut unsafe {
            self.mValue
                .mPairList
                .as_ref() // &*nsCSSValuePairList_heap
                .as_mut()
                .expect("List pointer should be non-null")
        }
        ._base as *mut structs::nsCSSValuePairList;
        while let Some(item) = unsafe { item_ptr.as_mut() } {
            let value = values.next().expect("Values shouldn't have been exhausted");
            item.mXValue = value.0;
            item.mYValue = value.1;
            item_ptr = item.mNext;
        }
        debug_assert!(values.next().is_none(), "Values should have been exhausted");
    }

    /// Set a shared list
    pub fn set_shared_list<I>(&mut self, values: I)
    where
        I: ExactSizeIterator<Item = nsCSSValue>,
    {
        debug_assert!(values.len() > 0, "Empty list is not supported");
        unsafe { bindings::Gecko_CSSValue_InitSharedList(self, values.len() as u32) };
        debug_assert_eq!(self.mUnit, nsCSSUnit::eCSSUnit_SharedList);
        let list = unsafe {
            self.mValue
                .mSharedList
                .as_ref()
                .as_mut()
                .expect("List pointer should be non-null")
                .mHead
                .as_mut()
        };
        debug_assert!(list.is_some(), "New created shared list shouldn't be null");
        for (item, new_value) in list.unwrap().into_iter().zip(values) {
            *item = new_value;
        }
    }
}

impl Drop for nsCSSValue {
    fn drop(&mut self) {
        unsafe { bindings::Gecko_CSSValue_Drop(self) };
    }
}

/// Iterator of nsCSSValueList.
#[allow(non_camel_case_types)]
pub struct nsCSSValueListIterator<'a> {
    current: Option<&'a nsCSSValueList>,
}

impl<'a> Iterator for nsCSSValueListIterator<'a> {
    type Item = &'a nsCSSValue;
    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            Some(item) => {
                self.current = unsafe { item.mNext.as_ref() };
                Some(&item.mValue)
            },
            None => None,
        }
    }
}

impl<'a> IntoIterator for &'a nsCSSValueList {
    type Item = &'a nsCSSValue;
    type IntoIter = nsCSSValueListIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        nsCSSValueListIterator {
            current: Some(self),
        }
    }
}

/// Mutable Iterator of nsCSSValueList.
#[allow(non_camel_case_types)]
pub struct nsCSSValueListMutIterator<'a> {
    current: *mut nsCSSValueList,
    phantom: PhantomData<&'a mut nsCSSValue>,
}

impl<'a> Iterator for nsCSSValueListMutIterator<'a> {
    type Item = &'a mut nsCSSValue;
    fn next(&mut self) -> Option<Self::Item> {
        match unsafe { self.current.as_mut() } {
            Some(item) => {
                self.current = item.mNext;
                Some(&mut item.mValue)
            },
            None => None,
        }
    }
}

impl<'a> IntoIterator for &'a mut nsCSSValueList {
    type Item = &'a mut nsCSSValue;
    type IntoIter = nsCSSValueListMutIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        nsCSSValueListMutIterator {
            current: self as *mut nsCSSValueList,
            phantom: PhantomData,
        }
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
    fn convert(self, nscssvalue: &mut nsCSSValue);
}

impl<T: ToNsCssValue> From<T> for nsCSSValue {
    fn from(value: T) -> nsCSSValue {
        let mut result = nsCSSValue::null();
        value.convert(&mut result);
        result
    }
}
