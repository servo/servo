// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.equals
description: >
  A property bag missing optional properties is equivalent to a property bag
  with all the optional properties having their default values
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 987, 654, 321);

const minimumProperties = {
  year: 2021,
  month: 10,
  day: 28,
};
const allProperties = {
  year: 2021,
  month: 10,
  day: 28,
  hour: 0,
  minute: 0,
  second: 0,
  millisecond: 0,
  microsecond: 0,
  nanosecond: 0,
  calendar: "iso8601",
};
const resultWithout = instance.equals(minimumProperties);
const resultWith = instance.equals(allProperties);
assert.sameValue(resultWithout, resultWith, "results should be the same with and without optional properties");
