// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth
description: PlainYearMonth constructor with invalid iso dates
features: [Temporal]
---*/

const tests = [
  [2020, 0, 24],
  [2020, 13, 24],
  [2020, -3, 24],
  [2020, 12, 32],
  [2020, 2, 30],
  [2019, 2, 29],
  [2019, 2, 0],
  [2019, 2, -20],
];

for (const [year, month, day] of tests) {
  assert.throws(RangeError, () => new Temporal.PlainYearMonth(year, month, undefined, day),
    `year=${year}, month=${month}, day=${day}`);
}
