// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.zoneddatetime.prototype.daysinmonth
description: Checking days in month for a "normal" case (non-undefined, non-boundary case, etc.)
features: [Temporal]
---*/

const tests = [
  [1976, 2, 18, 29],
  [1976, 11, 18, 30],
  [1976, 12, 18, 31],
  [1977, 2, 18, 28],
  [1997, 1, 23, 31],
  [1996, 2, 23, 29],
  [2000, 2, 23, 29],
  [1997, 2, 23, 28],
  [1997, 3, 23, 31],
  [1997, 4, 23, 30],
  [1997, 5, 23, 31],
  [1997, 6, 23, 30],
  [1997, 7, 23, 31],
  [1997, 8, 23, 31],
  [1997, 9, 23, 30],
  [1997, 10, 23, 31],
  [1997, 11, 23, 30],
  [1997, 12, 23, 31],
];
for (const [y, m, d, expected] of tests) {
  const plainDateTime = new Temporal.PlainDateTime(y, m, d, 15, 23, 30, 123, 456, 789);
  assert.sameValue(plainDateTime.toZonedDateTime("UTC").daysInMonth, expected, `${expected} days in the month of ${plainDateTime}`);
}


