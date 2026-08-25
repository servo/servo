// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.until
description: >
  A property bag missing optional properties is equivalent to a property bag
  with all the optional properties having their default values
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainTime(12, 34, 56, 987, 654, 321);

const minimumProperties = {
  hour: 0,
};
const allProperties = {
  hour: 0,
  minute: 0,
  second: 0,
  millisecond: 0,
  microsecond: 0,
  nanosecond: 0,
};
const resultWithout = instance.until(minimumProperties);
const resultWith = instance.until(allProperties);
TemporalHelpers.assertDurationsEqual(resultWithout, resultWith, "results should be the same with and without optional properties");
