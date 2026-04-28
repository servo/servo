// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: >
  Appropriate error thrown when argument cannot be converted to a valid string
  for time zone
features: [BigInt, Symbol, Temporal]
---*/

const instance = new Temporal.Duration(1);

const primitiveTests = [
  [null, "null"],
  [true, "boolean"],
  ["", "empty string"],
  [1, "number that doesn't convert to a valid ISO string"],
  [19761118, "number that would convert to a valid ISO string in other contexts"],
  [1n, "bigint"],
];

for (const [timeZone, description] of primitiveTests) {
  assert.throws(
    typeof timeZone === 'string' ? RangeError : TypeError,
    () => instance.total({ unit: "months", relativeTo: { year: 2000, month: 5, day: 2, timeZone } }),
    `${description} does not convert to a valid ISO string`
  );
}

const typeErrorTests = [
  [Symbol(), "symbol"],
  [{}, "object"],
  [new Temporal.Duration(), "duration instance"],
];

for (const [timeZone, description] of typeErrorTests) {
  assert.throws(TypeError, () => instance.total({ unit: "months", relativeTo: { year: 2000, month: 5, day: 2, timeZone } }), `${description} is not a valid object and does not convert to a string`);
}
