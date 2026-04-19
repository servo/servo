// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: The calendar name is case-insensitive
features: [Temporal]
---*/

const timeZone = "UTC";
const arg = { year: 1970, monthCode: "M01", day: 1, timeZone, calendar: "IsO8601" };
const result = Temporal.ZonedDateTime.from(arg);
assert.sameValue(result.calendarId, "iso8601", "Calendar is case-insensitive");

arg.calendar = "\u0130SO8601";
assert.throws(
  RangeError,
  () => Temporal.ZonedDateTime.from(arg),
  "calendar ID is capital dotted I is not lowercased"
);
