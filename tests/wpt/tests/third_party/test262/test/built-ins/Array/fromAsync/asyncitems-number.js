// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  Array.fromAsync treats a Number as an array-like
includes: [asyncHelpers.js, compareArray.js]
flags: [async]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
  Number.prototype.length = 2;
  Number.prototype[0] = 1;
  Number.prototype[1] = 2;

  const result = await Array.fromAsync(1);
  assert.compareArray(result, [1, 2]);
});
