/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use structs::{nsStyleCoord_CalcValue, nsStyleCoord_Calc, nsStyleUnit, nsStyleUnion};
use bindings::{Gecko_ResetStyleCoord, Gecko_SetStyleCoordCalcValue};

// Functions here are unsafe because it is possible to use the wrong nsStyleUnit
// FIXME we should be pairing up nsStyleUnion and nsStyleUnit somehow
// nsStyleCoord is one way to do it, but there are other structs using pairs
// of union and unit too

impl nsStyleUnion {
    /// Clean up any resources used by an nsStyleUnit
    /// Currently, this only happens if the nsStyleUnit
    /// is a Calc
    pub unsafe fn reset(&mut self, unit: &mut nsStyleUnit) {
        Gecko_ResetStyleCoord(unit, self);
    }

    /// Set internal value to a calc() value
    pub unsafe fn set_calc_value(&mut self, unit: &mut nsStyleUnit, v: nsStyleCoord_CalcValue) {
        Gecko_SetStyleCoordCalcValue(unit, self, v);
    }

    pub unsafe fn get_calc(&self) -> nsStyleCoord_CalcValue {
        (*(*self.mPointer.as_ref() as *const nsStyleCoord_Calc))._base
    }
}
