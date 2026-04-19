// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.equals
description: >
  Appropriate error thrown when argument cannot be converted to a valid string
  or property bag for PlainYearMonth
features: [BigInt, Symbol, Temporal]
---*/

const instance = new Temporal.PlainYearMonth(2000, 5);

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
  [Temporal.PlainYearMonth, "Temporal.PlainYearMonth, object"],
  [Temporal.PlainYearMonth.prototype, "Temporal.PlainYearMonth.prototype, object"],
];

for (const [arg, description] of typeErrorTests) {
  assert.throws(TypeError, () => instance.equals(arg), `${description} is not a valid property bag and does not convert to a string`);
}
