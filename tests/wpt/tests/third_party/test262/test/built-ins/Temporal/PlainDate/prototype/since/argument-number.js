// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.since
description: A number cannot be used in place of a Temporal.PlainDate
features: [Temporal]
---*/

const instance = new Temporal.PlainDate(1976, 11, 18);

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
    'Numbers cannot be used in place of an ISO string for PlainDate'
  );
}
