// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.withplaintime
description: A number is invalid in place of an ISO string for Temporal.PlainTime
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(2000, 5, 2);

const numbers = [
  1,
  -123456.987654321,
  1234567,
  123456.9876543219,
];

for (const arg of numbers) {
  assert.throws(
    TypeError,
    () => instance.withPlainTime(arg),
    `A number (${arg}) is not a valid ISO string for PlainTime`
  );
}
