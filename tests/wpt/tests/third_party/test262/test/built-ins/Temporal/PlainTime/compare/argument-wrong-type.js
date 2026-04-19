// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.compare
description: >
  Appropriate error thrown when argument cannot be converted to a valid string
  or property bag for PlainTime
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
    () => Temporal.PlainTime.compare(arg, new Temporal.PlainTime(12, 34, 56, 987, 654, 321)),
    `${description} does not convert to a valid ISO string (first argument)`
  );
  assert.throws(
    typeof arg === 'string' ? RangeError : TypeError,
    () => Temporal.PlainTime.compare(new Temporal.PlainTime(12, 34, 56, 987, 654, 321), arg),
    `${description} does not convert to a valid ISO string (second argument)`
  );
}

const typeErrorTests = [
  [Symbol(), "symbol"],
  [{}, "plain object"],
  [Temporal.PlainTime, "Temporal.PlainTime, object"],
  [Temporal.PlainTime.prototype, "Temporal.PlainTime.prototype, object"],
];

for (const [arg, description] of typeErrorTests) {
  assert.throws(TypeError, () => Temporal.PlainTime.compare(arg, new Temporal.PlainTime(12, 34, 56, 987, 654, 321)), `${description} is not a valid property bag and does not convert to a string (first argument)`);
  assert.throws(TypeError, () => Temporal.PlainTime.compare(new Temporal.PlainTime(12, 34, 56, 987, 654, 321), arg), `${description} is not a valid property bag and does not convert to a string (second argument)`);
}
