/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Rust helpers for Gecko's `nsStyleCoord`.

use crate::gecko_bindings::bindings;
use crate::gecko_bindings::structs::nsStyleSides;
use crate::gecko_bindings::structs::{nsStyleCoord, nsStyleCoord_Calc, nsStyleCoord_CalcValue};
use crate::gecko_bindings::structs::{nsStyleUnion, nsStyleUnit, nscoord};
use std::mem;

impl nsStyleCoord {
    #[inline]
    /// Get a `null` nsStyleCoord.
    pub fn null() -> Self {
        // Can't construct directly because it has private fields
        let mut coord: Self = unsafe { mem::zeroed() };
        coord.leaky_set_null();
        coord
    }
}

unsafe impl CoordData for nsStyleCoord {
    #[inline]
    fn unit(&self) -> nsStyleUnit {
        unsafe { *self.get_mUnit() }
    }
    #[inline]
    fn union(&self) -> nsStyleUnion {
        unsafe { *self.get_mValue() }
    }
}

unsafe impl CoordDataMut for nsStyleCoord {
    unsafe fn values_mut(&mut self) -> (&mut nsStyleUnit, &mut nsStyleUnion) {
        let unit = self.get_mUnit_mut() as *mut _;
        let value = self.get_mValue_mut() as *mut _;
        (&mut *unit, &mut *value)
    }
}

impl nsStyleCoord_CalcValue {
    /// Create an "empty" CalcValue (whose value is `0`).
    pub fn new() -> Self {
        nsStyleCoord_CalcValue {
            mLength: 0,
            mPercent: 0.0,
            mHasPercent: false,
        }
    }
}

impl PartialEq for nsStyleCoord_CalcValue {
    fn eq(&self, other: &Self) -> bool {
        self.mLength == other.mLength &&
            self.mPercent == other.mPercent &&
            self.mHasPercent == other.mHasPercent
    }
}

impl nsStyleSides {
    /// Immutably get the `nsStyleCoord`-like object representing the side at
    /// index `index`.
    #[inline]
    pub fn data_at(&self, index: usize) -> SidesData {
        SidesData {
            sides: self,
            index: index,
        }
    }

    /// Mutably get the `nsStyleCoord`-like object representing the side at
    /// index `index`.
    #[inline]
    pub fn data_at_mut(&mut self, index: usize) -> SidesDataMut {
        SidesDataMut {
            sides: self,
            index: index,
        }
    }
}

/// A `nsStyleCoord`-like object on top of an immutable reference to
/// `nsStyleSides`.
pub struct SidesData<'a> {
    sides: &'a nsStyleSides,
    index: usize,
}

/// A `nsStyleCoord`-like object on top of an mutable reference to
/// `nsStyleSides`.
pub struct SidesDataMut<'a> {
    sides: &'a mut nsStyleSides,
    index: usize,
}

unsafe impl<'a> CoordData for SidesData<'a> {
    #[inline]
    fn unit(&self) -> nsStyleUnit {
        unsafe { self.sides.get_mUnits()[self.index] }
    }
    #[inline]
    fn union(&self) -> nsStyleUnion {
        unsafe { self.sides.get_mValues()[self.index] }
    }
}
unsafe impl<'a> CoordData for SidesDataMut<'a> {
    #[inline]
    fn unit(&self) -> nsStyleUnit {
        unsafe { self.sides.get_mUnits()[self.index] }
    }
    #[inline]
    fn union(&self) -> nsStyleUnion {
        unsafe { self.sides.get_mValues()[self.index] }
    }
}
unsafe impl<'a> CoordDataMut for SidesDataMut<'a> {
    unsafe fn values_mut(&mut self) -> (&mut nsStyleUnit, &mut nsStyleUnion) {
        let unit = &mut self.sides.get_mUnits_mut()[self.index] as *mut _;
        let value = &mut self.sides.get_mValues_mut()[self.index] as *mut _;
        (&mut *unit, &mut *value)
    }
}

/// Enum representing the tagged union that is CoordData.
///
/// In release mode this should never actually exist in the code, and will be
/// optimized out by threading matches and inlining.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CoordDataValue {
    /// eStyleUnit_Null
    Null,
    /// eStyleUnit_Normal
    Normal,
    /// eStyleUnit_Auto
    Auto,
    /// eStyleUnit_None
    None,
    /// eStyleUnit_Percent
    Percent(f32),
    /// eStyleUnit_Factor
    Factor(f32),
    /// eStyleUnit_Degree
    Degree(f32),
    /// eStyleUnit_FlexFraction
    FlexFraction(f32),
    /// eStyleUnit_Coord
    Coord(nscoord),
    /// eStyleUnit_Integer
    Integer(i32),
    /// eStyleUnit_Enumerated
    Enumerated(u32),
    /// eStyleUnit_Calc
    Calc(nsStyleCoord_CalcValue),
}

/// A trait to abstract on top of a mutable `nsStyleCoord`-like object.
pub unsafe trait CoordDataMut: CoordData {
    /// Get mutably the unit and the union.
    ///
    /// This is unsafe since it's possible to modify the unit without changing
    /// the union.
    ///
    /// NB: This can't be two methods since we can't mutably borrow twice
    unsafe fn values_mut(&mut self) -> (&mut nsStyleUnit, &mut nsStyleUnion);

    /// Clean up any resources used by the union.
    ///
    /// Currently, this only happens if the nsStyleUnit is a Calc.
    #[inline]
    fn reset(&mut self) {
        unsafe {
            if self.unit() == nsStyleUnit::eStyleUnit_Calc {
                let (unit, union) = self.values_mut();
                bindings::Gecko_ResetStyleCoord(unit, union);
            }
        }
    }

    #[inline]
    /// Copies the unit and value from another `CoordData` type.
    fn copy_from<T: CoordData>(&mut self, other: &T) {
        unsafe {
            self.reset();
            self.copy_from_unchecked(other);
            self.addref_if_calc();
        }
    }

    #[inline]
    /// Moves the unit and value from another `CoordData` type.
    fn move_from<T: CoordData>(&mut self, other: T) {
        unsafe {
            self.reset();
            self.copy_from_unchecked(&other);
        }
    }

    #[inline]
    /// Copies the unit and value from another `CoordData` type without checking
    /// the type of the value (so refcounted values like calc may leak).
    unsafe fn copy_from_unchecked<T: CoordData>(&mut self, other: &T) {
        let (unit, union) = self.values_mut();
        *unit = other.unit();
        *union = other.union();
    }

    /// Useful for initializing uninits, given that `set_value` may segfault on
    /// uninits.
    fn leaky_set_null(&mut self) {
        use crate::gecko_bindings::structs::nsStyleUnit::*;
        unsafe {
            let (unit, union) = self.values_mut();
            *unit = eStyleUnit_Null;
            *union.mInt.as_mut() = 0;
        }
    }

    #[inline(always)]
    /// Sets the inner value.
    fn set_value(&mut self, value: CoordDataValue) {
        use self::CoordDataValue::*;
        use crate::gecko_bindings::structs::nsStyleUnit::*;
        self.reset();
        unsafe {
            let (unit, union) = self.values_mut();
            match value {
                Null => {
                    *unit = eStyleUnit_Null;
                    *union.mInt.as_mut() = 0;
                },
                Normal => {
                    *unit = eStyleUnit_Normal;
                    *union.mInt.as_mut() = 0;
                },
                Auto => {
                    *unit = eStyleUnit_Auto;
                    *union.mInt.as_mut() = 0;
                },
                None => {
                    *unit = eStyleUnit_None;
                    *union.mInt.as_mut() = 0;
                },
                Percent(f) => {
                    *unit = eStyleUnit_Percent;
                    *union.mFloat.as_mut() = f;
                },
                Factor(f) => {
                    *unit = eStyleUnit_Factor;
                    *union.mFloat.as_mut() = f;
                },
                Degree(f) => {
                    *unit = eStyleUnit_Degree;
                    *union.mFloat.as_mut() = f;
                },
                FlexFraction(f) => {
                    *unit = eStyleUnit_FlexFraction;
                    *union.mFloat.as_mut() = f;
                },
                Coord(coord) => {
                    *unit = eStyleUnit_Coord;
                    *union.mInt.as_mut() = coord;
                },
                Integer(i) => {
                    *unit = eStyleUnit_Integer;
                    *union.mInt.as_mut() = i;
                },
                Enumerated(i) => {
                    *unit = eStyleUnit_Enumerated;
                    *union.mInt.as_mut() = i as i32;
                },
                Calc(calc) => {
                    // Gecko_SetStyleCoordCalcValue changes the unit internally
                    bindings::Gecko_SetStyleCoordCalcValue(unit, union, calc);
                },
            }
        }
    }

    #[inline]
    /// Gets the `Calc` value mutably, asserts in debug builds if the unit is
    /// not `Calc`.
    unsafe fn as_calc_mut(&mut self) -> &mut nsStyleCoord_Calc {
        debug_assert_eq!(self.unit(), nsStyleUnit::eStyleUnit_Calc);
        &mut *(*self.union().mPointer.as_mut() as *mut nsStyleCoord_Calc)
    }

    #[inline]
    /// Does what it promises, if the unit is `calc`, it bumps the reference
    /// count _of the calc expression_.
    fn addref_if_calc(&mut self) {
        unsafe {
            if self.unit() == nsStyleUnit::eStyleUnit_Calc {
                bindings::Gecko_AddRefCalcArbitraryThread(self.as_calc_mut());
            }
        }
    }
}
/// A trait to abstract on top of a `nsStyleCoord`-like object.
pub unsafe trait CoordData {
    /// Get the unit of this object.
    fn unit(&self) -> nsStyleUnit;
    /// Get the `nsStyleUnion` for this object.
    fn union(&self) -> nsStyleUnion;

    #[inline(always)]
    /// Get the appropriate value for this object.
    fn as_value(&self) -> CoordDataValue {
        use self::CoordDataValue::*;
        use crate::gecko_bindings::structs::nsStyleUnit::*;
        unsafe {
            match self.unit() {
                eStyleUnit_Null => Null,
                eStyleUnit_Normal => Normal,
                eStyleUnit_Auto => Auto,
                eStyleUnit_None => None,
                eStyleUnit_Percent => Percent(self.get_float()),
                eStyleUnit_Factor => Factor(self.get_float()),
                eStyleUnit_Degree => Degree(self.get_float()),
                eStyleUnit_FlexFraction => FlexFraction(self.get_float()),
                eStyleUnit_Coord => Coord(self.get_integer()),
                eStyleUnit_Integer => Integer(self.get_integer()),
                eStyleUnit_Enumerated => Enumerated(self.get_integer() as u32),
                eStyleUnit_Calc => Calc(self.get_calc_value()),
            }
        }
    }

    #[inline]
    /// Pretend inner value is a float; obtain it.
    unsafe fn get_float(&self) -> f32 {
        use crate::gecko_bindings::structs::nsStyleUnit::*;
        debug_assert!(
            self.unit() == eStyleUnit_Percent ||
                self.unit() == eStyleUnit_Factor ||
                self.unit() == eStyleUnit_Degree ||
                self.unit() == eStyleUnit_FlexFraction
        );
        *self.union().mFloat.as_ref()
    }

    #[inline]
    /// Pretend inner value is an int; obtain it.
    unsafe fn get_integer(&self) -> i32 {
        use crate::gecko_bindings::structs::nsStyleUnit::*;
        debug_assert!(
            self.unit() == eStyleUnit_Coord ||
                self.unit() == eStyleUnit_Integer ||
                self.unit() == eStyleUnit_Enumerated
        );
        *self.union().mInt.as_ref()
    }

    #[inline]
    /// Pretend inner value is a calc; obtain it.
    /// Ensure that the unit is Calc before calling this.
    unsafe fn get_calc_value(&self) -> nsStyleCoord_CalcValue {
        debug_assert_eq!(self.unit(), nsStyleUnit::eStyleUnit_Calc);
        (*self.as_calc())._base
    }

    #[inline]
    /// Pretend the inner value is a calc expression, and obtain it.
    unsafe fn as_calc(&self) -> &nsStyleCoord_Calc {
        debug_assert_eq!(self.unit(), nsStyleUnit::eStyleUnit_Calc);
        &*(*self.union().mPointer.as_ref() as *const nsStyleCoord_Calc)
    }
}
