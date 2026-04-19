// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.round
description: Rounding increment should properly divide the relevant time unit
features: [Temporal]
---*/

const dt = new Temporal.PlainDateTime(1976, 11, 18, 14, 23, 30, 123, 456, 789);

[1, 2, 3, 4, 6, 8, 12].forEach((roundingIncrement) => {
  assert.sameValue(
    dt.round({ smallestUnit: "hour", roundingIncrement }) instanceof Temporal.PlainDateTime,
    true,
    `valid hour increments divide into 24 (rounding increment = ${roundingIncrement})`);
});

["minute", "second"].forEach((smallestUnit) => {
  [1, 2, 3, 4, 5, 6, 10, 12, 15, 20, 30].forEach((roundingIncrement) => {
    assert.sameValue(
      dt.round({ smallestUnit, roundingIncrement }) instanceof Temporal.PlainDateTime,
      true,
      `valid ${smallestUnit} increments divide into 60 (rounding increment = ${roundingIncrement})`
      );
  });
});

["millisecond", "microsecond", "nanosecond"].forEach((smallestUnit) => {
  [1, 2, 4, 5, 8, 10, 20, 25, 40, 50, 100, 125, 200, 250, 500].forEach((roundingIncrement) => {
    assert.sameValue(
      dt.round({ smallestUnit, roundingIncrement }) instanceof Temporal.PlainDateTime,
      true,
      `valid ${smallestUnit} increments divide into 1000 (rounding increment = ${roundingIncrement})`);
  });
});

const nextIncrements = {
  "hour": 24,
  "minute": 60,
  "second": 60,
  "millisecond": 1000,
  "microsecond": 1000,
  "nanosecond": 1000
};

Object.entries(nextIncrements).forEach(([unit, next]) => {
  assert.throws(
    RangeError,
    () => dt.round({ smallestUnit: unit, roundingIncrement: next }),
    `throws on increments that are equal to the next highest (unit = ${unit}, increment = ${next})`
  );
});
