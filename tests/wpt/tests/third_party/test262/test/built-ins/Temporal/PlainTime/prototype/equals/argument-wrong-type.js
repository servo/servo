// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.equals
description: >
  Appropriate error thrown when argument cannot be converted to a valid string
  or property bag for PlainTime
features: [BigInt, Symbol, Temporal]
---*/

const instance = new Temporal.PlainTime(12, 34, 56, 987, 654, 321);

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
    () => instance.equals(arg),
    `${description} does not convert to a valid ISO string`
  );
}

const typeErrorTests = [
  [Symbol(), "symbol"],
  [{}, "plain object"],
  [Temporal.PlainTime, "Temporal.PlainTime, object"],
  [Temporal.PlainTime.prototype, "Temporal.PlainTime.prototype, object"],
];

for (const [arg, description] of typeErrorTests) {
  assert.throws(TypeError, () => instance.equals(arg), `${description} is not a valid property bag and does not convert to a string`);
}
