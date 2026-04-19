// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: Valid rounding increments (based on divisibility).
features: [Temporal]
---*/

/*
const earlier = Temporal.ZonedDateTime.from('2019-01-08T09:22:36.123456789+01:00[+01:00]');
const later = Temporal.ZonedDateTime.from('2021-09-07T13:39:40.987654321+01:00[+01:00]');
*/
const earlier = new Temporal.ZonedDateTime(1546935756123456789n, "+01:00");
const later = new Temporal.ZonedDateTime(1631018380987654321n, "+01:00");

// valid hour increments divide into 24
[
  1,
  2,
  3,
  4,
  6,
  8,
  12
].forEach(roundingIncrement => {
  const options = {
    smallestUnit: "hours",
    roundingIncrement
  };
  assert(later.since(earlier, options) instanceof Temporal.Duration);
});
[
  "minutes",
  "seconds"
].forEach(smallestUnit => {
  [
    1,
    2,
    3,
    4,
    5,
    6,
    10,
    12,
    15,
    20,
    30
  ].forEach(roundingIncrement => {
    const options = {
      smallestUnit,
      roundingIncrement
    };
    assert(later.since(earlier, options) instanceof Temporal.Duration);
  });
});
[
  "milliseconds",
  "microseconds",
  "nanoseconds"
].forEach(smallestUnit => {
  [
    1,
    2,
    4,
    5,
    8,
    10,
    20,
    25,
    40,
    50,
    100,
    125,
    200,
    250,
    500
  ].forEach(roundingIncrement => {
    const options = {
      smallestUnit,
      roundingIncrement
    };
    assert(later.since(earlier, options) instanceof Temporal.Duration);
  });
});
