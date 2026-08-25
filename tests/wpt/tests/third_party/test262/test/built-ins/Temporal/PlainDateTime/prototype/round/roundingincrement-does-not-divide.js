// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.round
description: Throw exception if the rounding unit does not properly divide the relevant time unit
features: [Temporal]
---*/

const dt = new Temporal.PlainDateTime(1976, 11, 18, 14, 23, 30, 123, 456, 789);
const units = ["day", "hour", "minute", "second", "millisecond", "microsecond", "nanosecond"];
units.forEach((unit) => {
  assert.throws(
    RangeError,
    () => dt.round({ smallestUnit: unit, roundingIncrement: 29 }),
    `throws on increments that do not divide evenly into the next highest (unit = ${unit})`
  );
});
