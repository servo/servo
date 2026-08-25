// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.from
description: Appropriate error thrown if primitive input cannot convert to a valid string
features: [Temporal]
---*/

assert.throws(TypeError, () => Temporal.Duration.from(), "no argument");

const primitiveTests = [
  [undefined, "undefined"],
  [null, "null"],
  [true, "boolean"],
  [1, "number that doesn't convert to a valid ISO string"],
  [19761118, "number that would convert to a valid ISO string in other contexts"],
  [1n, "bigint"],
  [Symbol(), "symbol"],
  [{}, "plain object"],
  [Temporal.Duration, "Temporal.Duration, object"],
  [Temporal.Duration.prototype, "Temporal.Duration.prototype, object"],
];

for (const [arg, description] of primitiveTests) {
  assert.throws(
    TypeError,
    () => Temporal.Duration.from(arg),
    `${description} does not convert to a valid ISO string`
  );

  for (const options of [undefined, { overflow: 'constrain' }, { overflow: 'reject' }]) {
    assert.throws(
      TypeError,
      () => Temporal.Duration.from(arg, options),
      `${description} does not convert to a valid ISO string with options ${options}`
    );
  }
}
