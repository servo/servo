/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(crate) mod largestcontentfulpaint;
#[allow(clippy::module_inception, reason = "The interface name is Performance")]
pub(crate) mod performance;
#[expect(dead_code)]
pub(crate) mod performanceentry;
pub(crate) mod performancemark;
pub(crate) mod performancemeasure;
pub(crate) mod performancenavigation;
pub(crate) mod performancenavigationtiming;
#[expect(dead_code)]
pub(crate) mod performanceobserver;
pub(crate) mod performanceobserverentrylist;
pub(crate) mod performancepainttiming;
pub(crate) mod performanceresourcetiming;
pub(crate) use self::performance::Performance;

/// <https://w3c.github.io/navigation-timing/#the-performancetiming-interface>
///
/// Note: This interface is obselete and use of name is supported to remain backwards compatible.
pub(crate) const PERFORMANCE_TIMING_ATTRIBUTES: &[&str] = &[
    "navigationStart",
    "unloadEventStart",
    "unloadEventEnd",
    "redirectStart",
    "redirectEnd",
    "fetchStart",
    "domainLookupStart",
    "domainLookupEnd",
    "connectStart",
    "connectEnd",
    "secureConnectionStart",
    "requestStart",
    "responseStart",
    "responseEnd",
    "domLoading",
    "domInteractive",
    "domContentLoadedEventStart",
    "domContentLoadedEventEnd",
    "domComplete",
    "loadEventStart",
    "loadEventEnd",
];
