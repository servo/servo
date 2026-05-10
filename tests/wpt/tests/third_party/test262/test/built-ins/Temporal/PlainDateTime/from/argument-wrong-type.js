// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: >
  Appropriate error thrown when argument cannot be converted to a valid string
  or property bag for PlainDateTime
features: [BigInt, Symbol, Temporal]
---*/

assert.throws(TypeError, () => Temporal.PlainDateTime.from(), "no argument");

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
    () => Temporal.PlainDateTime.from(arg),
    `${description} does not convert to a valid ISO string`
  );

  for (const options of [undefined, { overflow: 'constrain' }, { overflow: 'reject' }]) {
    assert.throws(
      typeof arg === 'string' ? RangeError : TypeError,
      () => Temporal.PlainDateTime.from(arg, options),
      `${description} does not convert to a valid ISO string with options ${options}`
    );
  }
}

const typeErrorTests = [
  [Symbol(), "symbol"],
  [{}, "plain object"],
  [Temporal.PlainDateTime, "Temporal.PlainDateTime, object"],
  [Temporal.PlainDateTime.prototype, "Temporal.PlainDateTime.prototype, object"],
];

for (const [arg, description] of typeErrorTests) {
  assert.throws(TypeError, () => Temporal.PlainDateTime.from(arg), `${description} is not a valid property bag and does not convert to a string`);

  for (const options of [undefined, { overflow: 'constrain' }, { overflow: 'reject' }]) {
    assert.throws(TypeError, () => Temporal.PlainDateTime.from(arg, options), `${description} is not a valid property bag and does not convert to a string with options ${options}`);
  }
}
