// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Various forms of property bag passed to from()
features: [Temporal]
includes: [temporalHelpers.js]
---*/

TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 1976, month: 11, monthCode: "M11", day: 18, timeZone: "UTC" }).toPlainDateTime(),
  1976, 11, "M11", 18, 0, 0, 0, 0, 0, 0,
  "plain object with month & month code"
);

assert.throws(
  TypeError,
  () => Temporal.ZonedDateTime.from({}),
  "empty object throws"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 1976, month: 11, day: 18, millisecond: 123, timeZone: "UTC" }).toPlainDateTime(),
  1976, 11, "M11", 18, 0, 0, 0, 123, 0, 0,
  "plain object with month but not month code"
);

TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 1976, monthCode: "M09", day: 18, millisecond: 123, timeZone: "UTC" }).toPlainDateTime(),
  1976, 9, "M09", 18, 0, 0, 0, 123, 0, 0,
  "plain object with month code but not month"
);


TemporalHelpers.assertPlainDateTime(
  Temporal.ZonedDateTime.from({ year: 1976, month: 11, day: 18, hours: 12, timeZone: "UTC" }).toPlainDateTime(),
  1976, 11, "M11", 18, 0, 0, 0, 0, 0, 0,
  "incorrectly-spelled properties (e.g., plural \"hours\") are ignored"
);
