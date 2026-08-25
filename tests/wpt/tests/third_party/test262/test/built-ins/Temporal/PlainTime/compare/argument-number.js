// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.compare
description: A number is invalid in place of an ISO string for Temporal.PlainTime
features: [Temporal]
---*/

const numbers = [
  1,
  -123456.987654321,
  1234567,
  123456.9876543219,
];

for (const arg of numbers) {
  assert.throws(
    TypeError,
    () => Temporal.PlainTime.compare(arg, new Temporal.PlainTime(12, 34, 56, 987, 654, 321)),
    `A number (${arg}) is not a valid ISO string for PlainTime (first argument)`
  );
  assert.throws(
    TypeError,
    () => Temporal.PlainTime.compare(new Temporal.PlainTime(12, 34, 56, 987, 654, 321), arg),
    `A number (${arg}) is not a valid ISO string for PlainTime (second argument)`
  );
}
