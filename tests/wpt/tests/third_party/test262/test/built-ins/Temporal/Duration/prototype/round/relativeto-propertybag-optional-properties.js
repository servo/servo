// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.round
description: >
  A property bag missing optional properties is equivalent to a property bag
  with all the optional properties having their default values
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const timeZone = "UTC";
const instance = new Temporal.Duration(1, 0, 0, 0, 24);

let relativeTo = {
  year: 2021,
  month: 10,
  day: 28,
  timeZone,
};
const resultWithout = instance.round({ largestUnit: "years", relativeTo });
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
  timeZone,
  calendar: "iso8601",
};
const resultWith = instance.round({ largestUnit: "years", relativeTo });
TemporalHelpers.assertDurationsEqual(resultWithout, resultWith, "results should be the same with and without optional properties");
