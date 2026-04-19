// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: Test valid rounding increments (correct divisibility).
features: [Temporal]
---*/

const d = new Temporal.Duration(5, 5, 5, 5, 5, 5, 5, 5, 5, 5);
const relativeTo = new Temporal.PlainDate(2020, 1, 1);

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
    roundingIncrement,
    relativeTo
  };
  assert(d.round(options) instanceof Temporal.Duration);
});
[
  "minutes",
  "seconds"
].forEach(smallestUnit => {
  // valid minutes/seconds increments divide into 60
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
      const roundTo = {
        smallestUnit,
        roundingIncrement,
        relativeTo
      };
      assert(d.round(roundTo) instanceof Temporal.Duration);
    });
  });
[
  "milliseconds",
  "microseconds",
  "nanoseconds"
].forEach(smallestUnit => {
  // valid milliseconds/microseconds/nanoseconds increments divide into 1000
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
      const roundTo = {
        smallestUnit,
        roundingIncrement,
        relativeTo
      };
      assert(d.round(roundTo) instanceof Temporal.Duration);
    });
  });
