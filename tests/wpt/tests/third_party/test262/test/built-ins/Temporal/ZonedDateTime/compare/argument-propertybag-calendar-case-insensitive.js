// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: The calendar name is case-insensitive
features: [Temporal]
---*/

const timeZone = "UTC";
const datetime = new Temporal.ZonedDateTime(0n, timeZone);

const arg = { year: 1970, monthCode: "M01", day: 1, timeZone, calendar: "IsO8601" };
const result1 = Temporal.ZonedDateTime.compare(arg, datetime);
assert.sameValue(result1, 0, "Calendar is case-insensitive (first argument)");
const result2 = Temporal.ZonedDateTime.compare(datetime, arg);
assert.sameValue(result2, 0, "Calendar is case-insensitive (second argument)");

arg.calendar = "\u0130SO8601";
assert.throws(
  RangeError,
  () => Temporal.ZonedDateTime.compare(arg, datetime),
  "calendar ID is capital dotted I is not lowercased (first argument)"
);
assert.throws(
  RangeError,
  () => Temporal.ZonedDateTime.compare(datetime, arg),
  "calendar ID is capital dotted I is not lowercased (second argument)"
);
