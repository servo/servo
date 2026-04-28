// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  Array.fromAsync treats a Symbol as an array-like
includes: [asyncHelpers.js, compareArray.js]
flags: [async]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
  Symbol.prototype.length = 2;
  Symbol.prototype[0] = 1;
  Symbol.prototype[1] = 2;

  const result = await Array.fromAsync(Symbol());
  assert.compareArray(result, [1, 2]);
});
