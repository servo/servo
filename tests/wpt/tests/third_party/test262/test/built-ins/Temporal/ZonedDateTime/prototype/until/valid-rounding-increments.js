// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: Valid increments are based on divisibility.
features: [Temporal]
---*/

const earlier = new Temporal.ZonedDateTime(0n, "+01:00");
const later = new Temporal.ZonedDateTime(1_000_000_000_000_000_000n, "+01:00");

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
  var options = {
    smallestUnit: "hours",
    roundingIncrement
  };
  assert(earlier.until(later, options) instanceof Temporal.Duration);
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
    var options = {
      smallestUnit,
      roundingIncrement
    };
    assert(earlier.until(later, options) instanceof Temporal.Duration);
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
    var options = {
      smallestUnit,
      roundingIncrement
    };
    assert(earlier.until(later, options) instanceof Temporal.Duration);
  });
});
