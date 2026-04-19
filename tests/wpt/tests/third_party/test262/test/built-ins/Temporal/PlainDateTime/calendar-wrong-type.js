// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime
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
    () => new Temporal.PlainDateTime(2000, 5, 2, 15, 23, 30, 987, 654, 321, arg),
    `${description} is not a valid calendar`
  );
}
