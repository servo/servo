/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use net_traits::{ResourceAttribute, ResourceFetchTiming, ResourceTimeValue, ResourceTimingType};

#[test]
fn test_set_start_time_to_fetch_start_if_nonzero_tao() {
    let mut resource_timing: ResourceFetchTiming =
        ResourceFetchTiming::new(ResourceTimingType::Resource);
    resource_timing.fetch_start = 1;
    assert_eq!(resource_timing.start_time, 0, "`start_time` should be zero");
    assert!(
        resource_timing.fetch_start > 0,
        "`fetch_start` should have a positive value"
    );

    // verify that setting `start_time` to `fetch_start` succeeds
    resource_timing.set_attribute(ResourceAttribute::StartTime(ResourceTimeValue::FetchStart));
    assert_eq!(
        resource_timing.start_time, resource_timing.fetch_start,
        "`start_time` should equal `fetch_start`"
    );
}

#[test]
fn test_set_start_time_to_fetch_start_if_zero_tao() {
    let mut resource_timing: ResourceFetchTiming =
        ResourceFetchTiming::new(ResourceTimingType::Resource);
    resource_timing.start_time = 1;
    assert!(
        resource_timing.start_time > 0,
        "`start_time` should have a positive value"
    );
    assert_eq!(
        resource_timing.fetch_start, 0,
        "`fetch_start` should be zero"
    );

    // verify that setting `start_time` to `fetch_start` succeeds even when `fetch_start` == zero
    resource_timing.set_attribute(ResourceAttribute::StartTime(ResourceTimeValue::FetchStart));
    assert_eq!(
        resource_timing.start_time, resource_timing.fetch_start,
        "`start_time` should equal `fetch_start`"
    );
}

#[test]
fn test_set_start_time_to_fetch_start_if_nonzero_no_tao() {
    let mut resource_timing: ResourceFetchTiming =
        ResourceFetchTiming::new(ResourceTimingType::Resource);
    resource_timing.mark_timing_check_failed();
    resource_timing.fetch_start = 1;
    assert_eq!(resource_timing.start_time, 0, "`start_time` should be zero");
    assert!(
        resource_timing.fetch_start > 0,
        "`fetch_start` should have a positive value"
    );

    // verify that setting `start_time` to `fetch_start` succeeds even when TAO check failed
    resource_timing.set_attribute(ResourceAttribute::StartTime(ResourceTimeValue::FetchStart));
    assert_eq!(
        resource_timing.start_time, resource_timing.fetch_start,
        "`start_time` should equal `fetch_start`"
    );
}

#[test]
fn test_set_start_time_to_fetch_start_if_zero_no_tao() {
    let mut resource_timing: ResourceFetchTiming =
        ResourceFetchTiming::new(ResourceTimingType::Resource);
    resource_timing.mark_timing_check_failed();
    resource_timing.start_time = 1;
    assert!(
        resource_timing.start_time > 0,
        "`start_time` should have a positive value"
    );
    assert_eq!(
        resource_timing.fetch_start, 0,
        "`fetch_start` should be zero"
    );

    // verify that setting `start_time` to `fetch_start` succeeds even when `fetch_start`==0 and no TAO
    resource_timing.set_attribute(ResourceAttribute::StartTime(ResourceTimeValue::FetchStart));
    assert_eq!(
        resource_timing.start_time, resource_timing.fetch_start,
        "`start_time` should equal `fetch_start`"
    );
}

#[test]
fn test_set_start_time_to_redirect_start_if_nonzero_tao() {
    let mut resource_timing: ResourceFetchTiming =
        ResourceFetchTiming::new(ResourceTimingType::Resource);
    resource_timing.redirect_start = 1;
    assert_eq!(resource_timing.start_time, 0, "`start_time` should be zero");
    assert!(
        resource_timing.redirect_start > 0,
        "`redirect_start` should have a positive value"
    );

    // verify that setting `start_time` to `redirect_start` succeeds for nonzero `redirect_start`, TAO pass
    resource_timing.set_attribute(ResourceAttribute::StartTime(
        ResourceTimeValue::RedirectStart,
    ));
    assert_eq!(
        resource_timing.start_time, resource_timing.redirect_start,
        "`start_time` should equal `redirect_start`"
    );
}

#[test]
fn test_not_set_start_time_to_redirect_start_if_zero_tao() {
    let mut resource_timing: ResourceFetchTiming =
        ResourceFetchTiming::new(ResourceTimingType::Resource);
    resource_timing.start_time = 1;
    assert!(
        resource_timing.start_time > 0,
        "`start_time` should have a positive value"
    );
    assert_eq!(
        resource_timing.redirect_start, 0,
        "`redirect_start` should be zero"
    );

    // verify that setting `start_time` to `redirect_start` fails if `redirect_start` == 0
    resource_timing.set_attribute(ResourceAttribute::StartTime(
        ResourceTimeValue::RedirectStart,
    ));
    assert_ne!(
        resource_timing.start_time, resource_timing.redirect_start,
        "`start_time` should *not* equal `redirect_start`"
    );
}

#[test]
fn test_not_set_start_time_to_redirect_start_if_nonzero_no_tao() {
    let mut resource_timing: ResourceFetchTiming =
        ResourceFetchTiming::new(ResourceTimingType::Resource);
    resource_timing.mark_timing_check_failed();
    // Note: properly-behaved redirect_start should never be nonzero once TAO check has failed
    resource_timing.redirect_start = 1;
    assert_eq!(resource_timing.start_time, 0, "`start_time` should be zero");
    assert!(
        resource_timing.redirect_start > 0,
        "`redirect_start` should have a positive value"
    );

    // verify that setting `start_time` to `redirect_start` fails if TAO check fails
    resource_timing.set_attribute(ResourceAttribute::StartTime(
        ResourceTimeValue::RedirectStart,
    ));
    assert_ne!(
        resource_timing.start_time, resource_timing.redirect_start,
        "`start_time` should *not* equal `redirect_start`"
    );
}

#[test]
fn test_not_set_start_time_to_redirect_start_if_zero_no_tao() {
    let mut resource_timing: ResourceFetchTiming =
        ResourceFetchTiming::new(ResourceTimingType::Resource);
    resource_timing.mark_timing_check_failed();
    resource_timing.start_time = 1;
    assert!(
        resource_timing.start_time > 0,
        "`start_time` should have a positive value"
    );
    assert_eq!(
        resource_timing.redirect_start, 0,
        "`redirect_start` should be zero"
    );

    // verify that setting `start_time` to `redirect_start` fails if `redirect_start`==0 and no TAO
    resource_timing.set_attribute(ResourceAttribute::StartTime(
        ResourceTimeValue::RedirectStart,
    ));
    assert_ne!(
        resource_timing.start_time, resource_timing.redirect_start,
        "`start_time` should *not* equal `redirect_start`"
    );
}

#[test]
fn test_set_start_time() {
    let mut resource_timing: ResourceFetchTiming =
        ResourceFetchTiming::new(ResourceTimingType::Resource);
    assert_eq!(resource_timing.start_time, 0, "`start_time` should be zero");

    // verify setting `start_time` to current time succeeds
    resource_timing.set_attribute(ResourceAttribute::StartTime(ResourceTimeValue::Now));
    assert!(resource_timing.start_time > 0, "failed to set `start_time`");
}
#[test]
fn test_reset_start_time() {
    let mut resource_timing: ResourceFetchTiming =
        ResourceFetchTiming::new(ResourceTimingType::Resource);
    assert_eq!(resource_timing.start_time, 0, "`start_time` should be zero");

    resource_timing.start_time = 1;
    assert!(
        resource_timing.start_time > 0,
        "`start_time` should have a positive value"
    );

    // verify resetting `start_time` (to zero) succeeds
    resource_timing.set_attribute(ResourceAttribute::StartTime(ResourceTimeValue::Zero));
    assert_eq!(
        resource_timing.start_time, 0,
        "failed to reset `start_time`"
    );
}
