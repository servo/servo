// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate
description: PlainDate constructor with invalid iso dates
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
  assert.throws(RangeError, () => new Temporal.PlainDate(year, month, day),
    `year=${year}, month=${month}, day=${day}`);
}
