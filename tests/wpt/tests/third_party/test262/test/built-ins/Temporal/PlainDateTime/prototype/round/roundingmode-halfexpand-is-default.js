// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.round
description: Half-expand is the default rounding mode
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const dt = new Temporal.PlainDateTime(1976, 11, 18, 14, 23, 30, 123, 456, 789);

const units = {
          "day": [1976, 11, "M11", 19,  0,  0,  0,   0,   0,   0],
         "hour": [1976, 11, "M11", 18, 14,  0,  0,   0,   0,   0],
       "minute": [1976, 11, "M11", 18, 14, 24,  0,   0,   0,   0],
       "second": [1976, 11, "M11", 18, 14, 23, 30,   0,   0,   0],
  "millisecond": [1976, 11, "M11", 18, 14, 23, 30, 123,   0,   0],
  "microsecond": [1976, 11, "M11", 18, 14, 23, 30, 123, 457,   0],
   "nanosecond": [1976, 11, "M11", 18, 14, 23, 30, 123, 456, 789]
};

const expected = [1976, 11, "M11", 18, 0, 0, 0, 0, 0, 0];

Object.entries(units).forEach(([unit, expected]) => {
  TemporalHelpers.assertPlainDateTime(
    dt.round({ smallestUnit: unit }),
    ...expected,
    `halfExpand is the default (smallest unit = ${unit}, rounding mode absent)`
  );
});
