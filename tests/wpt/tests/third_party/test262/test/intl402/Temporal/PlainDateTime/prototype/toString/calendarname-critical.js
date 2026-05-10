// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tostring
description: >
  If calendarName is "calendar", the calendar ID should be included and prefixed
  with "!".
features: [Temporal]
---*/

const tests = [
  [[], "1976-11-18T15:23:00[!u-ca=iso8601]", "built-in ISO"],
  [["gregory"], "1976-11-18T15:23:00[!u-ca=gregory]", "built-in Gregorian"],
];

for (const [args, expected, description] of tests) {
  const date = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 0, 0, 0, 0, ...args);
  const result = date.toString({ calendarName: "critical" });
  assert.sameValue(result, expected, `${description} calendar for calendarName = critical`);
}
