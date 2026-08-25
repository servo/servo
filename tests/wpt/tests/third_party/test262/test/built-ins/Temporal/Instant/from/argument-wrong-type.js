// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.from
description: >
  Appropriate error thrown when argument cannot be converted to a valid string
  for Instant
features: [BigInt, Symbol, Temporal]
---*/

assert.throws(TypeError, () => Temporal.Instant.from(), "no argument");

const primitiveTests = [
  [undefined, 'undefined'],
  [null, 'null'],
  [true, 'boolean'],
  [1, "number that doesn't convert to a valid ISO string"],
  [19761118, 'number that would convert to a valid ISO string in other contexts'],
  [1n, 'bigint'],
  [Symbol(), 'symbol'],
  [Temporal.Instant.prototype, 'Temporal.Instant.prototype (fails brand check)'],
];

for (const [arg, description] of primitiveTests) {
  assert.throws(
    TypeError,
    () => Temporal.Instant.from(arg),
    `${description} does not convert to a valid ISO string`
  );

  for (const options of [undefined, { overflow: 'constrain' }, { overflow: 'reject' }]) {
    assert.throws(
      TypeError,
      () => Temporal.Instant.from(arg, options),
      `${description} does not convert to a valid ISO string with options ${options}`
    );
  }
}
