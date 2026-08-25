// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.compare
description: A number is invalid in place of an ISO string for Temporal.PlainYearMonth
features: [Temporal]
---*/

const arg = 201906;

const numbers = [
  1,
  201906,
  -201906,
  1234567,
];

for (const arg of numbers) {
  assert.throws(
    TypeError,
    () => Temporal.PlainYearMonth.compare(arg, new Temporal.PlainYearMonth(2019, 6)),
    `A number (${arg}) is not a valid ISO string for PlainYearMonth (first argument)`
  );
  assert.throws(
    TypeError,
    () => Temporal.PlainYearMonth.compare(new Temporal.PlainYearMonth(2019, 6), arg),
    `A number (${arg}) is not a valid ISO string for PlainYearMonth (first argument)`
  );
}
