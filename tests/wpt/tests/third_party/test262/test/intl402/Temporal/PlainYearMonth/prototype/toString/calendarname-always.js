// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.tostring
description: If calendarName is "always", the calendar ID should be included.
features: [Temporal]
---*/

const tests = [
  [[], "2000-05-01[u-ca=iso8601]", "built-in ISO"],
  [["gregory"], "2000-05-01[u-ca=gregory]", "built-in Gregorian"],
];

for (const [args, expected, description] of tests) {
  const yearmonth = new Temporal.PlainYearMonth(2000, 5, ...args);
  const result = yearmonth.toString({ calendarName: "always" });
  assert.sameValue(result, expected, `${description} calendar for calendarName = always`);
}
