// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: The calendar name is case-insensitive
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const arg = { year: 1976, monthCode: "M11", day: 18, calendar: "IsO8601" };
const result = Temporal.PlainDateTime.from(arg);
TemporalHelpers.assertPlainDateTime(result, 1976, 11, "M11", 18, 0, 0, 0, 0, 0, 0, "Calendar is case-insensitive");

arg.calendar = "\u0130SO8601";
assert.throws(
  RangeError,
  () => Temporal.PlainDateTime.from(arg),
  "calendar ID is capital dotted I is not lowercased"
);
