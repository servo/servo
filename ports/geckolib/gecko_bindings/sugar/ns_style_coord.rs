/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bindings::{Gecko_ResetStyleCoord, Gecko_SetStyleCoordCalcValue, Gecko_AddRefCalcArbitraryThread};
use std::mem::transmute;
use std::marker::PhantomData;
use structs::{nsStyleCoord_Calc, nsStyleUnit, nsStyleUnion, nsStyleCoord, nsStyleSides, nsStyleCorners};
use structs::{nsStyleCoord_CalcValue, nscoord};

impl nsStyleCoord {
    #[inline]
    pub unsafe fn addref_if_calc(&mut self) {
        self.data().addref_if_calc();
    }

    #[inline]
    pub fn data(&self) -> CoordData {
        CoordData {
            union: &self.mValue as *const _ as *mut _,
            unit: &self.mUnit as *const _ as *mut _,
            _marker: PhantomData,
        }
    }
}

impl nsStyleSides {
    #[inline]
    pub fn data_at(&self, index: usize) -> CoordData {
        CoordData {
            union: &self.mValues[index] as *const _ as *mut _,
            unit: &self.mUnits[index] as *const _ as *mut _,
            _marker: PhantomData,
        }
    }
}

impl nsStyleCorners {
    #[inline]
    pub fn data_at(&self, index: usize) -> CoordData {
        CoordData {
            union: &self.mValues[index] as *const _ as *mut _,
            unit: &self.mUnits[index] as *const _ as *mut _,
            _marker: PhantomData,
        }
    }
}

#[derive(Copy, Clone)]
/// Enum representing the tagged union that is CoordData
/// In release mode this should never actually exist in the code,
/// and will be optimized out by threading matches and inlining
pub enum CoordDataValues {
    Null,
    Normal,
    Auto,
    None,
    Percent(f32),
    Factor(f32),
    Degree(f32),
    Grad(f32),
    Radian(f32),
    Turn(f32),
    FlexFraction(f32),
    Coord(nscoord),
    Integer(i32),
    Enumerated(u32),
    Calc(nsStyleCoord_CalcValue),
}

/// XXXManishearth should this be using Cell/UnsafeCell?
pub struct CoordData<'a> {
    union: *mut nsStyleUnion,
    unit: *mut nsStyleUnit,
    _marker: PhantomData<&'a mut ()>,
}

impl<'a> CoordData<'a> {
    /// Clean up any resources used by the union
    /// Currently, this only happens if the nsStyleUnit
    /// is a Calc
    #[inline]
    pub fn reset(&mut self) {
        unsafe {
            if *self.unit == nsStyleUnit::eStyleUnit_Calc {
                Gecko_ResetStyleCoord(self.unit, self.union);
            }
        }
    }

    #[inline]
    pub fn copy_from(&mut self, other: &CoordData) {
        self.reset();
        self.unit = other.unit;
        self.union = other.union;
        self.addref_if_calc();
    }

    #[inline(always)]
    pub fn as_enum(&self) -> CoordDataValues {
        use self::CoordDataValues::*;
        use structs::nsStyleUnit::*;
        unsafe {
            match *self.unit {
                eStyleUnit_Null => Null,
                eStyleUnit_Normal => Normal,
                eStyleUnit_Auto => Auto,
                eStyleUnit_None => None,
                eStyleUnit_Percent => Percent(self.get_float()),
                eStyleUnit_Factor => Factor(self.get_float()),
                eStyleUnit_Degree => Degree(self.get_float()),
                eStyleUnit_Grad => Grad(self.get_float()),
                eStyleUnit_Radian => Radian(self.get_float()),
                eStyleUnit_Turn => Turn(self.get_float()),
                eStyleUnit_FlexFraction => FlexFraction(self.get_float()),
                eStyleUnit_Coord => Coord(self.get_integer()),
                eStyleUnit_Integer => Integer(self.get_integer()),
                eStyleUnit_Enumerated => Enumerated(self.get_integer() as u32),
                eStyleUnit_Calc => Calc(self.get_calc()),
            }
        }
    }

    #[inline(always)]
    pub fn set_enum(&mut self, value: CoordDataValues) {
        use self::CoordDataValues::*;
        use structs::nsStyleUnit::*;
        self.reset();
        unsafe {
            match value {
                Null => {
                    *self.unit = eStyleUnit_Null;
                    *(*self.union).mInt.as_mut() = 0;
                }
                Normal => {
                    *self.unit = eStyleUnit_Normal;
                    *(*self.union).mInt.as_mut() = 0;
                }
                Auto => {
                    *self.unit = eStyleUnit_Auto;
                    *(*self.union).mInt.as_mut() = 0;
                }
                None => {
                    *self.unit = eStyleUnit_None;
                    *(*self.union).mInt.as_mut() = 0;
                }
                Percent(f) => {
                    *self.unit = eStyleUnit_Percent;
                    *(*self.union).mFloat.as_mut() = f;
                }
                Factor(f) => {
                    *self.unit = eStyleUnit_Factor;
                    *(*self.union).mFloat.as_mut() = f;
                }
                Degree(f) => {
                    *self.unit = eStyleUnit_Degree;
                    *(*self.union).mFloat.as_mut() = f;
                }
                Grad(f) => {
                    *self.unit = eStyleUnit_Grad;
                    *(*self.union).mFloat.as_mut() = f;
                }
                Radian(f) => {
                    *self.unit = eStyleUnit_Radian;
                    *(*self.union).mFloat.as_mut() = f;
                }
                Turn(f) => {
                    *self.unit = eStyleUnit_Turn;
                    *(*self.union).mFloat.as_mut() = f;
                }
                FlexFraction(f) => {
                    *self.unit = eStyleUnit_FlexFraction;
                    *(*self.union).mFloat.as_mut() = f;
                }
                Coord(coord) => {
                    *self.unit = eStyleUnit_Coord;
                    *(*self.union).mInt.as_mut() = coord;
                }
                Integer(i) => {
                    *self.unit = eStyleUnit_Integer;
                    *(*self.union).mInt.as_mut() = i;
                }
                Enumerated(i) => {
                    *self.unit = eStyleUnit_Enumerated;
                    *(*self.union).mInt.as_mut() = i as i32;
                }
                Calc(calc) => {
                    *self.unit = eStyleUnit_Calc;
                    self.set_calc_value(calc);
                }
            }
        }
    }

    #[inline]
    /// Pretend inner value is a float; obtain it
    /// While this should usually be called with the unit checked,
    /// it is not an intrinsically unsafe operation to call this function
    /// with the wrong unit
    pub fn get_float(&self) -> f32 {
        unsafe { *(*self.union).mFloat.as_ref() }
    }

    #[inline]
    /// Pretend inner value is an int; obtain it
    /// While this should usually be called with the unit checked,
    /// it is not an intrinsically unsafe operation to call this function
    /// with the wrong unit
    pub fn get_integer(&self) -> i32 {
        unsafe { *(*self.union).mInt.as_ref() }
    }

    #[inline]
    /// Pretend inner value is a calc; obtain it
    /// Ensure that the unit is Calc before calling this
    pub unsafe fn get_calc(&self) -> nsStyleCoord_CalcValue {
        (*self.as_calc())._base
    }

    /// Set internal value to a calc() value
    /// reset() the union before calling this
    #[inline]
    pub fn set_calc_value(&mut self, v: nsStyleCoord_CalcValue) {
        unsafe {
            // Calc should have been cleaned up
            debug_assert!(*self.unit != nsStyleUnit::eStyleUnit_Calc);
            Gecko_SetStyleCoordCalcValue(self.unit, self.union, v);
        }
    }

    #[inline]
    pub fn addref_if_calc(&mut self) {
        unsafe {
            if *self.unit == nsStyleUnit::eStyleUnit_Calc {
                Gecko_AddRefCalcArbitraryThread(self.as_calc_mut());
            }
        }
    }

    #[inline]
    unsafe fn as_calc_mut(&mut self) -> &mut nsStyleCoord_Calc {
        transmute(*(*self.union).mPointer.as_mut() as *mut nsStyleCoord_Calc)
    }

    #[inline]
    unsafe fn as_calc(&self) -> &nsStyleCoord_Calc {
        transmute(*(*self.union).mPointer.as_ref() as *const nsStyleCoord_Calc)
    }
}
