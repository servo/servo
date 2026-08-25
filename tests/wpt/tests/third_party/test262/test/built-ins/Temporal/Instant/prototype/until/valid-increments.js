// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.until
description: Test various rounding increments.
features: [Temporal]
---*/

const earlier = Temporal.Instant.from("1969-07-24T16:50:35.123456789Z");
const later = Temporal.Instant.from("2019-10-29T10:46:38.271986102Z");
const largestUnit = "hours";

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
    largestUnit,
    smallestUnit: "hours",
    roundingIncrement
  };
  assert(earlier.until(later, options) instanceof Temporal.Duration);
});

// valid increments divide into 60
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
      largestUnit,
      smallestUnit,
      roundingIncrement
    };
    assert(earlier.until(later, options) instanceof Temporal.Duration);
  });
});

// valid increments divide into 1000
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
      largestUnit,
      smallestUnit,
      roundingIncrement
    };
    assert(earlier.until(later, options) instanceof Temporal.Duration);
  });
});
