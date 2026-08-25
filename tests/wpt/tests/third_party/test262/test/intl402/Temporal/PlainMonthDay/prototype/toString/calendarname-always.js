// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.tostring
description: If calendarName is "always", the calendar ID should be included.
features: [Temporal]
---*/

const tests = [
  [[], "1972-05-02[u-ca=iso8601]", "built-in ISO"],
  [["gregory"], "1972-05-02[u-ca=gregory]", "built-in Gregorian"],
];

for (const [args, expected, description] of tests) {
  const monthday = new Temporal.PlainMonthDay(5, 2, ...args);
  const result = monthday.toString({ calendarName: "always" });
  assert.sameValue(result, expected, `${description} calendar for calendarName = always`);
}
