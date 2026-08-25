// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: >
  A property bag missing optional properties is equivalent to a property bag
  with all the optional properties having their default values
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const timeZone = "UTC";
const instance = new Temporal.ZonedDateTime(0n, timeZone);

const minimumProperties = {
  year: 2021,
  month: 10,
  day: 28,
  timeZone,
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
  timeZone,
  calendar: "iso8601",
};
const resultWithout = instance.until(minimumProperties);
const resultWith = instance.until(allProperties);
TemporalHelpers.assertDurationsEqual(resultWithout, resultWith, "results should be the same with and without optional properties");
