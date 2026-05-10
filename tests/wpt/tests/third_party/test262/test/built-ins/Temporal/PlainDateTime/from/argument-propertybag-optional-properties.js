// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: >
  A property bag missing optional properties is equivalent to a property bag
  with all the optional properties having their default values
features: [Temporal]
---*/

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
const resultWithout = Temporal.PlainDateTime.from(minimumProperties);
const resultWith = Temporal.PlainDateTime.from(allProperties);
assert(resultWithout.equals(resultWith), "results should be the same with and without optional properties");
