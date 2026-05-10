// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.compare
description: A number cannot be used in place of a Temporal.PlainDate
features: [Temporal]
---*/

const numbers = [
  1,
  19761118,
  -19761118,
  1234567890,
];

for (const arg of numbers) {
  assert.throws(
    TypeError,
    () => Temporal.PlainDate.compare(arg, new Temporal.PlainDate(1976, 11, 18)),
    "A number is not a valid ISO string for PlainDate (first argument)"
  );
  assert.throws(
    TypeError,
    () => Temporal.PlainDate.compare(new Temporal.PlainDate(1976, 11, 18), arg),
    "A number is not a valid ISO string for PlainDate (second argument)"
  );
}
