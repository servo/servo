// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.with
description: Non-object arguments throw.
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 987, 654, 321);
const args = [
  undefined,
  null,
  true,
  "2020-01-12T10:20:30",
  Symbol(),
  2020,
  2020n,
  NaN,
];
for (const argument of args) {
  assert.throws(TypeError, () => instance.with(argument), `Does not support ${typeof argument}`);
}
