// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.since
description: A number cannot be used in place of a Temporal.PlainDateTime
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(1976, 11, 18);

const numbers = [
  1,
  19761118,
  -19761118,
  1234567890,
];

for (const arg of numbers) {
  assert.throws(
    TypeError,
    () => instance.since(arg),
    `A number (${arg}) is not a valid ISO string for PlainDateTime`
  );
}
