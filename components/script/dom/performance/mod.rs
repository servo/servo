/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(crate) mod largestcontentfulpaint;
#[allow(clippy::module_inception, reason = "The interface name is Performance")]
pub(crate) mod performance;
#[allow(dead_code)]
pub(crate) mod performanceentry;
pub(crate) mod performancemark;
pub(crate) mod performancemeasure;
pub(crate) mod performancenavigation;
pub(crate) mod performancenavigationtiming;
#[allow(dead_code)]
pub(crate) mod performanceobserver;
pub(crate) mod performanceobserverentrylist;
pub(crate) mod performancepainttiming;
pub(crate) mod performanceresourcetiming;

pub(crate) use self::performance::Performance;
