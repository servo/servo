/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use bindings::{Gecko_ResetStyleCoord, Gecko_SetStyleCoordCalcValue, Gecko_AddRefCalcArbitraryThread};
use std::mem::transmute;
use structs::{nsStyleCoord_CalcValue, nsStyleCoord_Calc, nsStyleUnit, nsStyleUnion, nsStyleCoord};

// Functions here are unsafe because it is possible to use the wrong nsStyleUnit
// FIXME we should be pairing up nsStyleUnion and nsStyleUnit somehow
// nsStyleCoord is one way to do it, but there are other structs using pairs
// of union and unit too

impl nsStyleUnion {
    /// Clean up any resources used by an nsStyleUnit
    /// Currently, this only happens if the nsStyleUnit
    /// is a Calc
    pub unsafe fn reset(&mut self, unit: &mut nsStyleUnit) {
        if *unit == nsStyleUnit::eStyleUnit_Calc {
            Gecko_ResetStyleCoord(unit, self);
        }
    }

    /// Set internal value to a calc() value
    /// reset() the union before calling this
    pub unsafe fn set_calc_value(&mut self, unit: &mut nsStyleUnit, v: nsStyleCoord_CalcValue) {
        // Calc should have been cleaned up
        debug_assert!(*unit != nsStyleUnit::eStyleUnit_Calc);
        Gecko_SetStyleCoordCalcValue(unit, self, v);
    }

    pub unsafe fn get_calc(&self) -> nsStyleCoord_CalcValue {
        (*self.as_calc())._base
    }

    pub unsafe fn addref_if_calc(&mut self, unit: &nsStyleUnit) {
        if *unit == nsStyleUnit::eStyleUnit_Calc {
            Gecko_AddRefCalcArbitraryThread(self.as_calc_mut());
        }
    }

    unsafe fn as_calc_mut(&mut self) -> &mut nsStyleCoord_Calc {
        transmute(*self.mPointer.as_mut() as *mut nsStyleCoord_Calc)
    }
    unsafe fn as_calc(&self) -> &nsStyleCoord_Calc {
        transmute(*self.mPointer.as_ref() as *const nsStyleCoord_Calc)
    }
}

impl nsStyleCoord {
    pub unsafe fn addref_if_calc(&mut self) {
        self.mValue.addref_if_calc(&self.mUnit);
    }
}
