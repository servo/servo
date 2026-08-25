// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.round
description: String as first argument is equivalent to options bag with smallestUnit option
includes: [temporalHelpers.js]
features: [Temporal, arrow-function]
---*/

const instance = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 789, 999, 999);
const validUnits = [
  "day",
  "hour",
  "minute",
  "second",
  "millisecond",
  "microsecond",
  "nanosecond",
];
validUnits.forEach((smallestUnit) => {
  const full = instance.round({ smallestUnit });
  const shorthand = instance.round(smallestUnit);
  TemporalHelpers.assertPlainDateTimesEqual(shorthand, full, `"${smallestUnit}" as first argument to round is equivalent to options bag`);
});
