// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.equals
description: >
  Appropriate error thrown when argument cannot be converted to a valid string
  for Instant
features: [BigInt, Symbol, Temporal]
---*/

const instance = new Temporal.Instant(0n);

const primitiveTests = [
  [undefined, "undefined"],
  [null, "null"],
  [true, "boolean"],
  ["", "empty string"],
  [1, "number that doesn't convert to a valid ISO string"],
  [19761118, "number that would convert to a valid ISO string in other contexts"],
  [1n, "bigint"],
  [{}, "plain object"],
  [Temporal.Instant, "Temporal.Instant, object"],
];

for (const [arg, description] of primitiveTests) {
  assert.throws(
    typeof arg === "string" || (typeof arg === "object" && arg !== null) || typeof arg === "function"
      ? RangeError
      : TypeError,
    () => instance.equals(arg),
    `${description} does not convert to a valid ISO string`
  );
}

const typeErrorTests = [
  [Symbol(), "symbol"],
  [Temporal.Instant.prototype, "Temporal.Instant.prototype, object"],  // fails brand check in toString()
];

for (const [arg, description] of typeErrorTests) {
  assert.throws(TypeError, () => instance.equals(arg), `${description} does not convert to a string`);
}
