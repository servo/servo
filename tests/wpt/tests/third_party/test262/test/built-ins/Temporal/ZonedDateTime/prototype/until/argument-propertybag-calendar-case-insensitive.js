// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: The calendar name is case-insensitive
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const timeZone = "UTC";
const instance = new Temporal.ZonedDateTime(0n, timeZone);

const arg = { year: 1970, monthCode: "M01", day: 1, timeZone, calendar: "IsO8601" };
const result = instance.until(arg);
TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "Calendar is case-insensitive");

arg.calendar = "\u0130SO8601";
assert.throws(
  RangeError,
  () => instance.until(arg),
  "calendar ID is capital dotted I is not lowercased"
);
