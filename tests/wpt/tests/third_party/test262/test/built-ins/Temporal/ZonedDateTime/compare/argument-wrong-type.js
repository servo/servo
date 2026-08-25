// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: >
  Appropriate error thrown when argument cannot be converted to a valid string
  or property bag for ZonedDateTime
features: [BigInt, Symbol, Temporal]
---*/

const timeZone = "UTC";
const other = new Temporal.ZonedDateTime(0n, timeZone);

const primitiveTests = [
  [undefined, "undefined"],
  [null, "null"],
  [true, "boolean"],
  ["", "empty string"],
  [1, "number that doesn't convert to a valid ISO string"],
  [19761118, "number that would convert to a valid ISO string in other contexts"],
  [1n, "bigint"],
];

for (const [arg, description] of primitiveTests) {
  assert.throws(
    typeof arg === 'string' ? RangeError : TypeError,
    () => Temporal.ZonedDateTime.compare(arg, other),
    `${description} does not convert to a valid ISO string (first argument)`
  );
  assert.throws(
    typeof arg === 'string' ? RangeError : TypeError,
    () => Temporal.ZonedDateTime.compare(other, arg),
    `${description} does not convert to a valid ISO string (second argument)`
  );
}

const typeErrorTests = [
  [Symbol(), "symbol"],
  [{}, "plain object"],
  [Temporal.ZonedDateTime, "Temporal.ZonedDateTime, object"],
  [Temporal.ZonedDateTime.prototype, "Temporal.ZonedDateTime.prototype, object"],
];

for (const [arg, description] of typeErrorTests) {
  assert.throws(TypeError, () => Temporal.ZonedDateTime.compare(arg, other), `${description} is not a valid property bag and does not convert to a string (first argument)`);
  assert.throws(TypeError, () => Temporal.ZonedDateTime.compare(other, arg), `${description} is not a valid property bag and does not convert to a string (second argument)`);
}
