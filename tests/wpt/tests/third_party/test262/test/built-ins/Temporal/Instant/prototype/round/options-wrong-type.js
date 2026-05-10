// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.round
description: TypeError thrown when options argument is missing or a non-string primitive
features: [BigInt, Symbol, Temporal]
---*/

const badOptions = [
  undefined,
  null,
  true,
  Symbol(),
  1,
  2n,
];

const instance = new Temporal.Instant(0n);
assert.throws(TypeError, () => instance.round(), "TypeError on missing options argument");
for (const value of badOptions) {
  assert.throws(TypeError, () => instance.round(value),
    `TypeError on wrong options type ${typeof value}`);
};
