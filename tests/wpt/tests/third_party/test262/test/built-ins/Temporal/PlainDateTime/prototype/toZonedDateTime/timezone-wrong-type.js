// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tozoneddatetime
description: >
  Appropriate error thrown when argument cannot be converted to a valid string
  for time zone
features: [BigInt, Symbol, Temporal]
---*/

const instance = new Temporal.PlainDateTime(2000, 5, 2);

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
    () => instance.toZonedDateTime(timeZone),
    `${description} does not convert to a valid ISO string`
  );
}

const typeErrorTests = [
  [Symbol(), "symbol"],
  [{}, "object"],
  [new Temporal.Duration(), "duration instance"],
];

for (const [timeZone, description] of typeErrorTests) {
  assert.throws(TypeError, () => instance.toZonedDateTime(timeZone), `${description} is not a valid object and does not convert to a string`);
}
