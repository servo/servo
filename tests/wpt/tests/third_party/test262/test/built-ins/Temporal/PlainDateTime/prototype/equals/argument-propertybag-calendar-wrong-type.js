// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.equals
description: >
  Appropriate error thrown when a calendar property from a property bag cannot
  be converted to a calendar ID
features: [BigInt, Symbol, Temporal]
---*/

const instance = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 987, 654, 321);

const wrongTypeTests = [
  [null, "null"],
  [true, "boolean"],
  [1, "number"],
  [1n, "bigint"],
  [19970327, "large number"],
  [-19970327, "negative number"],
  [1234567890, "very large integer"],
  [Symbol(), "symbol"],
  [{}, "object"],
  [new Temporal.Duration(), "duration instance"],
];

for (const [calendar, description] of wrongTypeTests) {
  const arg = { year: 1976, monthCode: "M11", day: 18, calendar };
  assert.throws(
    TypeError,
    () => instance.equals(arg),
    `${description} is not a valid calendar`
  );
}
