// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.tostring
description: If calendarName is "auto", "iso8601" should be omitted.
features: [Temporal]
---*/

const tests = [
  [[], "2000-05", "built-in ISO"],
  [["gregory"], "2000-05-01[u-ca=gregory]", "built-in Gregorian"],
];

for (const [args, expected, description] of tests) {
  const yearmonth = new Temporal.PlainYearMonth(2000, 5, ...args);
  const result = yearmonth.toString({ calendarName: "auto" });
  assert.sameValue(result, expected, `${description} calendar for calendarName = auto`);
}
