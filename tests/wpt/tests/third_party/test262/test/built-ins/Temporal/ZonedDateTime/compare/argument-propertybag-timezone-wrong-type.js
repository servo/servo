// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: >
  Appropriate error thrown when argument cannot be converted to a valid string
  or object for TimeZone
features: [BigInt, Symbol, Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(0n, "UTC");

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
    () => Temporal.ZonedDateTime.compare({ year: 2020, month: 5, day: 2, timeZone }, datetime),
    `${description} does not convert to a valid ISO string (first argument)`
  );
  assert.throws(
    typeof timeZone === 'string' ? RangeError : TypeError,
    () => Temporal.ZonedDateTime.compare(datetime, { year: 2020, month: 5, day: 2, timeZone }),
    `${description} does not convert to a valid ISO string (second argument)`
  );
}

const typeErrorTests = [
  [Symbol(), "symbol"],
  [{}, "object"],
  [new Temporal.Duration(), "duration instance"],
];

for (const [timeZone, description] of typeErrorTests) {
  assert.throws(TypeError, () => Temporal.ZonedDateTime.compare({ year: 2020, month: 5, day: 2, timeZone }, datetime), `${description} is not a valid object and does not convert to a string (first argument)`);
  assert.throws(TypeError, () => Temporal.ZonedDateTime.compare(datetime, { year: 2020, month: 5, day: 2, timeZone }), `${description} is not a valid object and does not convert to a string (second argument)`);
}
