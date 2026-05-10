// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tostring
description: If calendarName is "never", the calendar ID should be omitted.
features: [Temporal]
---*/

const tests = [
  [[], "1976-11-18T15:23:00", "built-in ISO"],
  [["gregory"], "1976-11-18T15:23:00", "built-in Gregorian"],
];

for (const [args, expected, description] of tests) {
  const date = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 0, 0, 0, 0, ...args);
  const result = date.toString({ calendarName: "never" });
  assert.sameValue(result, expected, `${description} calendar for calendarName = never`);
}
