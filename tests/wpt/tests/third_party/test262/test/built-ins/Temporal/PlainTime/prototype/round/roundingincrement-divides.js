// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.round
description: Rounding increment should properly divide the relevant time unit
features: [Temporal]
---*/

const t = new Temporal.PlainTime(14, 23, 30, 123, 456, 789);

const minutesSeconds = [1, 2, 3, 4, 5, 6, 10, 12, 15, 20, 30];
const subSeconds = [1, 2, 4, 5, 8, 10, 20, 25, 40, 50, 100, 200, 250, 500];

const unitsAndIncrements = {
   "hour": [1, 2, 3, 4, 6, 8, 12],
   "minute": minutesSeconds,
   "second": minutesSeconds,
   "millisecond": subSeconds,
   "microsecond": subSeconds,
   "nanosecond": subSeconds,
};

// Just check that each combination of unit and increment doesn't throw
Object.entries(unitsAndIncrements).forEach(([unit, increments]) => {
  increments.forEach((increment) => {
    const result = t.round({ smallestUnit: unit, roundingMode: "ceil", roundingIncrement: increment });
    assert.sameValue(result instanceof Temporal.PlainTime, true, `${unit} ${increment}`);
  })
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
    () => t.round({ smallestUnit: unit, roundingIncrement: next }),
    `throws on increments that are equal to the next highest (unit = ${unit}, increment = ${next})`
  );
});
