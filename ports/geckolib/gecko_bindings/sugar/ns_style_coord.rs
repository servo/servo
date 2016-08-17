/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bindings::{Gecko_ResetStyleCoord, Gecko_SetStyleCoordCalcValue, Gecko_AddRefCalcArbitraryThread};
use structs::{nsStyleCoord_Calc, nsStyleUnit, nsStyleUnion, nsStyleCoord, nsStyleSides, nsStyleCorners};
use structs::{nsStyleCoord_CalcValue, nscoord};

impl CoordData for nsStyleCoord {
    #[inline]
    fn unit(&self) -> nsStyleUnit {
        unsafe {
            *self.get_mUnit()
        }
    }
    #[inline]
    fn union(&self) -> nsStyleUnion {
        unsafe {
            *self.get_mValue()
        }
    }
}

impl CoordDataMut for nsStyleCoord {
    unsafe fn values_mut(&mut self) -> (&mut nsStyleUnit, &mut nsStyleUnion) {
        let unit = self.get_mUnit_mut() as *mut _;
        let value = self.get_mValue_mut() as *mut _;
        (&mut *unit, &mut *value)
    }
}

impl nsStyleSides {
    #[inline]
    pub fn data_at(&self, index: usize) -> SidesData {
        SidesData {
            sides: self,
            index: index,
        }
    }
    #[inline]
    pub fn data_at_mut(&mut self, index: usize) -> SidesDataMut {
        SidesDataMut {
            sides: self,
            index: index,
        }
    }
}

pub struct SidesData<'a> {
    sides: &'a nsStyleSides,
    index: usize,
}
pub struct SidesDataMut<'a> {
    sides: &'a mut nsStyleSides,
    index: usize,
}

impl<'a> CoordData for SidesData<'a> {
    #[inline]
    fn unit(&self) -> nsStyleUnit {
        unsafe {
            self.sides.get_mUnits()[self.index]
        }
    }
    #[inline]
    fn union(&self) -> nsStyleUnion {
        unsafe {
            self.sides.get_mValues()[self.index]
        }
    }
}
impl<'a> CoordData for SidesDataMut<'a> {
    #[inline]
    fn unit(&self) -> nsStyleUnit {
        unsafe {
            self.sides.get_mUnits()[self.index]
        }
    }
    #[inline]
    fn union(&self) -> nsStyleUnion {
        unsafe {
            self.sides.get_mValues()[self.index]
        }
    }
}
impl<'a> CoordDataMut for SidesDataMut<'a> {
    unsafe fn values_mut(&mut self) -> (&mut nsStyleUnit, &mut nsStyleUnion) {
        let unit = &mut self.sides.get_mUnits_mut()[self.index] as *mut _;
        let value = &mut self.sides.get_mValues_mut()[self.index] as *mut _;
        (&mut *unit, &mut *value)
    }
}

impl nsStyleCorners {
    #[inline]
    pub fn data_at(&self, index: usize) -> CornersData {
        CornersData {
            corners: self,
            index: index,
        }
    }
    #[inline]
    pub fn data_at_mut(&mut self, index: usize) -> CornersDataMut {
        CornersDataMut {
            corners: self,
            index: index,
        }
    }
}

pub struct CornersData<'a> {
    corners: &'a nsStyleCorners,
    index: usize,
}
pub struct CornersDataMut<'a> {
    corners: &'a mut nsStyleCorners,
    index: usize,
}

impl<'a> CoordData for CornersData<'a> {
    fn unit(&self) -> nsStyleUnit {
        unsafe {
            self.corners.get_mUnits()[self.index]
        }
    }
    fn union(&self) -> nsStyleUnion {
        unsafe {
            self.corners.get_mValues()[self.index]
        }
    }
}
impl<'a> CoordData for CornersDataMut<'a> {
    fn unit(&self) -> nsStyleUnit {
        unsafe {
            self.corners.get_mUnits()[self.index]
        }
    }
    fn union(&self) -> nsStyleUnion {
        unsafe {
            self.corners.get_mValues()[self.index]
        }
    }
}
impl<'a> CoordDataMut for CornersDataMut<'a> {
    unsafe fn values_mut(&mut self) -> (&mut nsStyleUnit, &mut nsStyleUnion) {
        let unit = &mut self.corners.get_mUnits_mut()[self.index] as *mut _;
        let value = &mut self.corners.get_mValues_mut()[self.index] as *mut _;
        (&mut *unit, &mut *value)
    }
}

#[derive(Copy, Clone)]
/// Enum representing the tagged union that is CoordData.
/// In release mode this should never actually exist in the code,
/// and will be optimized out by threading matches and inlining.
pub enum CoordDataValue {
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


pub trait CoordDataMut : CoordData {
    // This can't be two methods since we can't mutably borrow twice
    /// This is unsafe since it's possible to modify
    /// the unit without changing the union
    unsafe fn values_mut(&mut self) -> (&mut nsStyleUnit, &mut nsStyleUnion);

    /// Clean up any resources used by the union
    /// Currently, this only happens if the nsStyleUnit
    /// is a Calc
    #[inline]
    fn reset(&mut self) {
        unsafe {
            if self.unit() == nsStyleUnit::eStyleUnit_Calc {
                let (unit, union) = self.values_mut();
                Gecko_ResetStyleCoord(unit, union);
            }
        }
    }

    #[inline]
    fn copy_from<T: CoordData>(&mut self, other: &T) {
        unsafe {
            self.reset();
            {
                let (unit, union) = self.values_mut();
                *unit = other.unit();
                *union = other.union();
            }
            self.addref_if_calc();
        }
    }

    #[inline]
    unsafe fn copy_from_unchecked<T: CoordData>(&mut self, other: &T) {
            let (unit, union) = self.values_mut();
            *unit = other.unit();
            *union = other.union();
    }

    /// Useful for initializing uninits
    /// (set_value may segfault on uninits)
    fn leaky_set_null(&mut self) {
        use structs::nsStyleUnit::*;
        unsafe {
            let (unit, union) = self.values_mut();
            *unit = eStyleUnit_Null;
            *union.mInt.as_mut() = 0;
        }
    }

    #[inline(always)]
    fn set_value(&mut self, value: CoordDataValue) {
        use self::CoordDataValue::*;
        use structs::nsStyleUnit::*;
        self.reset();
        unsafe {
            let (unit, union) = self.values_mut();
            match value {
                Null => {
                    *unit = eStyleUnit_Null;
                    *union.mInt.as_mut() = 0;
                }
                Normal => {
                    *unit = eStyleUnit_Normal;
                    *union.mInt.as_mut() = 0;
                }
                Auto => {
                    *unit = eStyleUnit_Auto;
                    *union.mInt.as_mut() = 0;
                }
                None => {
                    *unit = eStyleUnit_None;
                    *union.mInt.as_mut() = 0;
                }
                Percent(f) => {
                    *unit = eStyleUnit_Percent;
                    *union.mFloat.as_mut() = f;
                }
                Factor(f) => {
                    *unit = eStyleUnit_Factor;
                    *union.mFloat.as_mut() = f;
                }
                Degree(f) => {
                    *unit = eStyleUnit_Degree;
                    *union.mFloat.as_mut() = f;
                }
                Grad(f) => {
                    *unit = eStyleUnit_Grad;
                    *union.mFloat.as_mut() = f;
                }
                Radian(f) => {
                    *unit = eStyleUnit_Radian;
                    *union.mFloat.as_mut() = f;
                }
                Turn(f) => {
                    *unit = eStyleUnit_Turn;
                    *union.mFloat.as_mut() = f;
                }
                FlexFraction(f) => {
                    *unit = eStyleUnit_FlexFraction;
                    *union.mFloat.as_mut() = f;
                }
                Coord(coord) => {
                    *unit = eStyleUnit_Coord;
                    *union.mInt.as_mut() = coord;
                }
                Integer(i) => {
                    *unit = eStyleUnit_Integer;
                    *union.mInt.as_mut() = i;
                }
                Enumerated(i) => {
                    *unit = eStyleUnit_Enumerated;
                    *union.mInt.as_mut() = i as i32;
                }
                Calc(calc) => {
                    // Gecko_SetStyleCoordCalcValue changes the unit internally
                    Gecko_SetStyleCoordCalcValue(unit, union, calc);
                }
            }
        }
    }

    #[inline]
    unsafe fn as_calc_mut(&mut self) -> &mut nsStyleCoord_Calc {
        debug_assert!(self.unit() == nsStyleUnit::eStyleUnit_Calc);
        &mut *(*self.union().mPointer.as_mut() as *mut nsStyleCoord_Calc)
    }

    #[inline]
    fn addref_if_calc(&mut self) {
        unsafe {
            if self.unit() == nsStyleUnit::eStyleUnit_Calc {
                Gecko_AddRefCalcArbitraryThread(self.as_calc_mut());
            }
        }
    }
}
pub trait CoordData {
    fn unit(&self) -> nsStyleUnit;
    fn union(&self) -> nsStyleUnion;


    #[inline(always)]
    fn as_value(&self) -> CoordDataValue {
        use self::CoordDataValue::*;
        use structs::nsStyleUnit::*;
        unsafe {
            match self.unit() {
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
                eStyleUnit_Calc => Calc(self.get_calc_value()),
            }
        }
    }

    #[inline]
    /// Pretend inner value is a float; obtain it.
    unsafe fn get_float(&self) -> f32 {
        use structs::nsStyleUnit::*;
        debug_assert!(self.unit() == eStyleUnit_Percent || self.unit() == eStyleUnit_Factor
                      || self.unit() == eStyleUnit_Degree || self.unit() == eStyleUnit_Grad
                      || self.unit() == eStyleUnit_Radian || self.unit() == eStyleUnit_Turn
                      || self.unit() == eStyleUnit_FlexFraction);
        *self.union().mFloat.as_ref()
    }

    #[inline]
    /// Pretend inner value is an int; obtain it.
    unsafe fn get_integer(&self) -> i32 {
        use structs::nsStyleUnit::*;
        debug_assert!(self.unit() == eStyleUnit_Coord || self.unit() == eStyleUnit_Integer
                      || self.unit() == eStyleUnit_Enumerated);
        *self.union().mInt.as_ref()
    }

    #[inline]
    /// Pretend inner value is a calc; obtain it.
    /// Ensure that the unit is Calc before calling this.
    unsafe fn get_calc_value(&self) -> nsStyleCoord_CalcValue {
        debug_assert!(self.unit() == nsStyleUnit::eStyleUnit_Calc);
        (*self.as_calc())._base
    }


    #[inline]
    unsafe fn as_calc(&self) -> &nsStyleCoord_Calc {
        debug_assert!(self.unit() == nsStyleUnit::eStyleUnit_Calc);
        &*(*self.union().mPointer.as_ref() as *const nsStyleCoord_Calc)
    }
}
