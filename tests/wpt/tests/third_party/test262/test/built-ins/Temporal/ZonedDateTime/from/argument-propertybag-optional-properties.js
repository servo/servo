// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: >
  A property bag missing optional properties is equivalent to a property bag
  with all the optional properties having their default values
features: [Temporal]
---*/

const minimumProperties = {
  year: 2021,
  month: 10,
  day: 28,
  timeZone: "UTC",
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
  offset: "+00:00",
  timeZone: "UTC",
  calendar: "iso8601",
};
const resultWithout = Temporal.ZonedDateTime.from(minimumProperties);
const resultWith = Temporal.ZonedDateTime.from(allProperties);
assert(resultWithout.equals(resultWith), "results should be the same with and without optional properties");
