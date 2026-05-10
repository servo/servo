// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.since
description: >
  Appropriate error thrown when argument cannot be converted to a valid string
  or property bag for PlainDateTime
features: [BigInt, Symbol, Temporal]
---*/

const instance = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 987, 654, 321);

const primitiveTests = [
  [undefined, "undefined"],
  [null, "null"],
  [true, "boolean"],
  ["", "empty string"],
  [1, "number that doesn't convert to a valid ISO string"],
  [1n, "bigint"],
];

for (const [arg, description] of primitiveTests) {
  assert.throws(
    typeof arg === 'string' ? RangeError : TypeError,
    () => instance.since(arg),
    `${description} does not convert to a valid ISO string`
  );
}

const typeErrorTests = [
  [Symbol(), "symbol"],
  [{}, "plain object"],
  [Temporal.PlainDateTime, "Temporal.PlainDateTime, object"],
  [Temporal.PlainDateTime.prototype, "Temporal.PlainDateTime.prototype, object"],
];

for (const [arg, description] of typeErrorTests) {
  assert.throws(TypeError, () => instance.since(arg), `${description} is not a valid property bag and does not convert to a string`);
}
