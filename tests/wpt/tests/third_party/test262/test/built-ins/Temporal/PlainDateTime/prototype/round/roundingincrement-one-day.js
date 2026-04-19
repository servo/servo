// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.round
description: One day is a valid rounding increment
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const dt = new Temporal.PlainDateTime(1976, 11, 18, 14, 23, 30, 123, 456, 789);

TemporalHelpers.assertPlainDateTime(
  dt.round({ smallestUnit: "day", roundingIncrement: 1 }),
  1976, 11, "M11", 19, 0, 0, 0, 0, 0, 0,
  "1 day is a valid increment"
);
