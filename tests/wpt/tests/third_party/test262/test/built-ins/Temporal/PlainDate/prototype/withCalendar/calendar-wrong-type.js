// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.withcalendar
description: >
  Appropriate error thrown when argument cannot be converted to a valid string
  for Calendar
features: [BigInt, Symbol, Temporal]
---*/

const instance = new Temporal.PlainDate(1976, 11, 18, "iso8601");

const wrongTypeTests = [
  [null, "null"],
  [true, "boolean"],
  [1, "number"],
  [1n, "bigint"],
  [-19761118, "negative number"],
  [19761118, "large positive number"],
  [1234567890, "very large integer"],
  [Symbol(), "symbol"],
  [{}, "object"],
  [new Temporal.Duration(), "duration instance"],
];

for (const [arg, description] of wrongTypeTests) {
  assert.throws(
    TypeError,
    () => instance.withCalendar(arg),
    `${description} is not a valid calendar`
  );
}
