// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.since
description: Test valid increments.
features: [Temporal]
---*/

const earlier = Temporal.Instant.from("1976-11-18T15:23:30.123456789Z");
const later = Temporal.Instant.from("2019-10-29T10:46:38.271986102Z");
const largestUnit = "hours";

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
  assert(later.since(earlier, options) instanceof Temporal.Duration);
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
    assert(later.since(earlier, options) instanceof Temporal.Duration);
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
    assert(later.since(earlier, options) instanceof Temporal.Duration);
  });
});

