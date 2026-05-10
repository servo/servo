// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.compare
description: >
  A property bag missing optional properties is equivalent to a property bag
  with all the optional properties having their default values
features: [Temporal]
---*/

const duration1 = new Temporal.Duration(1);
const duration2 = new Temporal.Duration(0, 1);

let relativeTo = {
  year: 2021,
  month: 10,
  day: 28,
  timeZone: "UTC",
};
const resultWithout = Temporal.Duration.compare(duration1, duration2, { relativeTo });
relativeTo = {
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
const resultWith = Temporal.Duration.compare(duration1, duration2, { relativeTo });
assert.sameValue(resultWithout, resultWith, "results should be the same with and without optional properties");
