// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.compare
description: >
  Appropriate error thrown when argument cannot be converted to a valid string
  or property bag for PlainDateTime
features: [BigInt, Symbol, Temporal]
---*/

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
    () => Temporal.PlainDateTime.compare(arg, new Temporal.PlainDateTime(1976, 11, 18)),
    `${description} does not convert to a valid ISO string (first argument)`
  );
  assert.throws(
    typeof arg === 'string' ? RangeError : TypeError,
    () => Temporal.PlainDateTime.compare(new Temporal.PlainDateTime(1976, 11, 18), arg),
    `${description} does not convert to a valid ISO string (second argument)`
  );
}

const typeErrorTests = [
  [Symbol(), "symbol"],
  [{}, "plain object"],
  [Temporal.PlainDateTime, "Temporal.PlainDateTime, object"],
  [Temporal.PlainDateTime.prototype, "Temporal.PlainDateTime.prototype, object"],
];

for (const [arg, description] of typeErrorTests) {
  assert.throws(TypeError, () => Temporal.PlainDateTime.compare(arg, new Temporal.PlainDateTime(1976, 11, 18)), `${description} is not a valid property bag and does not convert to a string (first argument)`);
  assert.throws(TypeError, () => Temporal.PlainDateTime.compare(new Temporal.PlainDateTime(1976, 11, 18), arg), `${description} is not a valid property bag and does not convert to a string (second argument)`);
}
