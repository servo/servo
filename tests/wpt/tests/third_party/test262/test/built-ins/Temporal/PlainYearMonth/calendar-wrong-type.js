// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth
description: >
  Appropriate error thrown when argument cannot be converted to a valid string
  for Calendar
features: [BigInt, Symbol, Temporal]
---*/

const wrongTypeTests = [
  [null, "null"],
  [true, "boolean"],
  [1, "number"],
  [1n, "bigint"],
  [-19761118, "negative number"],
  [19761118, "large positive number"],
  [1234567890, "large integer"],
  [Symbol(), "symbol"],
  [{}, "object"],
  [new Temporal.Duration(), "duration instance"],
];

for (const [arg, description] of wrongTypeTests) {
  assert.throws(
    TypeError,
    () => new Temporal.PlainYearMonth(2000, 5, arg, 1),
    `${description} is not a valid calendar`
  );
}
