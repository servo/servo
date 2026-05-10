// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.tostring
description: TypeError thrown when options argument is a primitive
features: [BigInt, Symbol, Temporal]
---*/

const badOptions = [
  null,
  true,
  "some string",
  Symbol(),
  1,
  2n,
];

const instance = new Temporal.Duration(0, 0, 0, 0, 1);
for (const value of badOptions) {
  assert.throws(TypeError, () => instance.toString(value),
    `TypeError on wrong options type ${typeof value}`);
};
