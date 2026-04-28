// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: String as first argument is equivalent to options bag with smallestUnit option
includes: [temporalHelpers.js]
features: [Temporal, arrow-function]
---*/

const instance = new Temporal.Duration(0, 0, 0, 4, 5, 6, 7, 987, 654, 321);
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
  TemporalHelpers.assertDurationsEqual(shorthand, full, `"${smallestUnit}" as first argument to round is equivalent to options bag`);
});
