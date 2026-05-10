// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.tostring
description: If calendarName is "never", the calendar ID should be omitted.
features: [Temporal]
---*/

const tests = [
  [[], "2000-05", "built-in ISO"],
  [["gregory"], "2000-05-01", "built-in Gregorian"],
];

for (const [args, expected, description] of tests) {
  const yearmonth = new Temporal.PlainYearMonth(2000, 5, ...args);
  const result = yearmonth.toString({ calendarName: "never" });
  assert.sameValue(result, expected, `${description} calendar for calendarName = never`);
}
