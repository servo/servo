// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.plainyearmonth.prototype.daysinmonth
description: daysInMonth works
features: [Temporal]
---*/

const tests = [
  [new Temporal.PlainYearMonth(1976, 2), 29],
  [new Temporal.PlainYearMonth(1976, 11), 30],
  [new Temporal.PlainYearMonth(1976, 12), 31],
  [new Temporal.PlainYearMonth(1977, 2), 28],
];
for (const [plainYearMonth, expected] of tests) {
  assert.sameValue(plainYearMonth.daysInMonth, expected, `${expected} days in the month of ${plainYearMonth}`);
}
