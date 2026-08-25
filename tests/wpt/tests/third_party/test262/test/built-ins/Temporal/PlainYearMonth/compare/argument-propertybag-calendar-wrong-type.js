// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.compare
description: >
  Appropriate error thrown when a calendar property from a property bag cannot
  be converted to a calendar ID
features: [BigInt, Symbol, Temporal]
---*/

const wrongTypeTests = [
  [null, "null"],
  [true, "boolean"],
  [1, "number"],
  [1n, "bigint"],
  [19970327, "large number"],
  [-19970327, "negative number"],
  [1234567890, "very large integer"],
  [Symbol(), "symbol"],
  [{}, "object"],
  [new Temporal.Duration(), "duration instance"],
];

for (const [calendar, description] of wrongTypeTests) {
  const arg = { year: 2019, monthCode: "M06", calendar };
  assert.throws(
    TypeError,
    () => Temporal.PlainYearMonth.compare(arg, new Temporal.PlainYearMonth(2019, 6)),
    `${description} is not a valid calendar (first argument)`
  );
  assert.throws(
    TypeError,
    () => Temporal.PlainYearMonth.compare(new Temporal.PlainYearMonth(2019, 6), arg),
    `${description} is not a valid calendar (second argument)`
  );
}
