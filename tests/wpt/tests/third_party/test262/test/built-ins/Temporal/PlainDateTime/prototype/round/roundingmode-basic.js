// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.round
description: Basic checks for rounding mode
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const dt = new Temporal.PlainDateTime(1976, 11, 18, 14, 23, 30, 123, 456, 789);

TemporalHelpers.assertPlainDateTime(
  dt.round({ smallestUnit: "hour", roundingIncrement: 4 }),
  1976, 11, "M11", 18, 16, 0, 0, 0, 0, 0,
  "rounds to an increment of hours"
);

TemporalHelpers.assertPlainDateTime(
  dt.round({ smallestUnit: "minute", roundingIncrement: 15 }),
  1976, 11, "M11", 18, 14, 30, 0, 0, 0, 0,
  "rounds to an increment of minutes"
);

TemporalHelpers.assertPlainDateTime(
  dt.round({ smallestUnit: "second", roundingIncrement: 30 }),
  1976, 11, "M11", 18, 14, 23, 30, 0, 0, 0,
  "rounds to an increment of seconds"
);

TemporalHelpers.assertPlainDateTime(
  dt.round({ smallestUnit: "millisecond", roundingIncrement: 10 }),
  1976, 11, "M11", 18, 14, 23, 30, 120, 0, 0,
  "rounds to an increment of milliseconds"
);

TemporalHelpers.assertPlainDateTime(
  dt.round({ smallestUnit: "microsecond", roundingIncrement: 10 }),
  1976, 11, "M11", 18, 14, 23, 30, 123, 460, 0,
  "rounds to an increment of microseconds"
);

TemporalHelpers.assertPlainDateTime(
  dt.round({ smallestUnit: "nanosecond", roundingIncrement: 10 }),
  1976, 11, "M11", 18, 14, 23, 30, 123, 456, 790,
  "rounds to an increment of nanoseconds"
);
