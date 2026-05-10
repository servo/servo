// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: Rounding increments that do not cleanly divide the relevant unit
features: [Temporal]
---*/

const earlier = new Temporal.PlainDateTime(2019, 1, 8, 8, 22, 36, 123, 456, 789);
const later = new Temporal.PlainDateTime(2021, 9, 7, 12, 39, 40, 987, 654, 321);

const nondivisibleUnits = {
  "hours": 11,
  "minutes": 29,
  "seconds": 29,
  "milliseconds": 29,
  "microseconds": 29,
  "nanoseconds": 29
};

Object.entries(nondivisibleUnits).forEach(([unit, increment]) => {
  assert.throws(
    RangeError,
    () => earlier.until(later, {smallestUnit: unit, roundingIncrement: increment}),
    `throws on increments that do not divide evenly into the next highest (unit = ${unit}, increment = ${increment})`
  );
});

const equalDivisibleUnits = {
  "hours": 24,
  "minutes": 60,
  "seconds": 60,
  "milliseconds": 1000,
  "microseconds": 1000,
  "nanoseconds": 1000
};

Object.entries(equalDivisibleUnits).forEach(([unit, increment]) => {
  assert.throws(
    RangeError,
    () => earlier.until(later, {smallestUnit: unit, roundingIncrement: increment}),
    `throws on increments that are equal to the next highest (unit = ${unit}, rounding increment = ${increment})`
  );
});
