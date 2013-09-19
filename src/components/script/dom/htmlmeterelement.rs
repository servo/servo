/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::ErrorResult;
use dom::htmlelement::HTMLElement;

pub struct HTMLMeterElement {
    htmlelement: HTMLElement
}

impl HTMLMeterElement {
    pub fn Value(&self) -> f64 {
        0.0
    }

    pub fn SetValue(&mut self, _value: f64) -> ErrorResult {
        Ok(())
    }

    pub fn Min(&self) -> f64 {
        0.0
    }

    pub fn SetMin(&mut self, _min: f64) -> ErrorResult {
        Ok(())
    }

    pub fn Max(&self) -> f64 {
        0.0
    }

    pub fn SetMax(&mut self, _max: f64) -> ErrorResult {
        Ok(())
    }

    pub fn Low(&self) -> f64 {
        0.0
    }

    pub fn SetLow(&mut self, _low: f64) -> ErrorResult {
        Ok(())
    }

    pub fn High(&self) -> f64 {
        0.0
    }

    pub fn SetHigh(&mut self, _high: f64) -> ErrorResult {
        Ok(())
    }

    pub fn Optimum(&self) -> f64 {
        0.0
    }

    pub fn SetOptimum(&mut self, _optimum: f64) -> ErrorResult {
        Ok(())
    }
}
