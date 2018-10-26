/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use script::test::timeranges::TimeRangesContainer;

fn check(time_ranges: &TimeRangesContainer, expected: &'static str) {
    assert_eq!(
        format!("{:?}", time_ranges),
        format!("TimeRangesContainer {{ ranges: [{}] }}", expected)
    );
}

#[test]
fn initial_state() {
    let time_ranges = TimeRangesContainer::new();
    assert_eq!(time_ranges.len(), 0);
    assert!(time_ranges.start(0).is_err());
    assert!(time_ranges.end(0).is_err());
}

#[test]
fn error_if_start_is_older_than_end() {
    let mut time_ranges = TimeRangesContainer::new();
    assert!(time_ranges.add(2., 1.).is_err());
}

#[test]
fn single_range() {
    let mut time_ranges = TimeRangesContainer::new();
    time_ranges.add(1., 2.).unwrap();
    check(&time_ranges, "[1,2)");
    assert_eq!(time_ranges.start(0).unwrap(), 1.);
    assert_eq!(time_ranges.end(0).unwrap(), 2.);
}

#[test]
fn add_order() {
    let mut time_ranges_a = TimeRangesContainer::new();
    for range in vec![(0., 2.), (3., 4.), (5., 100.)].iter() {
        time_ranges_a.add(range.0, range.1).unwrap();
    }
    let expected = "[0,2), [3,4), [5,100)";
    check(&time_ranges_a, expected);

    let mut time_ranges_b = TimeRangesContainer::new();
    // Add the values in time_ranges_a to time_ranges_b in reverse order.
    for i in (0..time_ranges_a.len()).rev() {
        time_ranges_b
            .add(
                time_ranges_a.start(i).unwrap(),
                time_ranges_a.end(i).unwrap(),
            )
            .unwrap();
    }
    check(&time_ranges_b, expected);
}

#[test]
fn add_overlapping() {
    let mut time_ranges = TimeRangesContainer::new();

    time_ranges.add(0., 2.).unwrap();
    time_ranges.add(10., 11.).unwrap();
    check(&time_ranges, "[0,2), [10,11)");

    time_ranges.add(0., 2.).unwrap();
    check(&time_ranges, "[0,2), [10,11)");

    time_ranges.add(2., 3.).unwrap();
    check(&time_ranges, "[0,3), [10,11)");

    time_ranges.add(2., 6.).unwrap();
    check(&time_ranges, "[0,6), [10,11)");

    time_ranges.add(9., 10.).unwrap();
    check(&time_ranges, "[0,6), [9,11)");

    time_ranges.add(8., 10.).unwrap();
    check(&time_ranges, "[0,6), [8,11)");

    time_ranges.add(-1., 7.).unwrap();
    check(&time_ranges, "[-1,7), [8,11)");

    time_ranges.add(6., 9.).unwrap();
    check(&time_ranges, "[-1,11)");
}
