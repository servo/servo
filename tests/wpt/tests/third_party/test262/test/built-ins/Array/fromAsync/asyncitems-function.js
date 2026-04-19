// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  Array.fromAsync treats a function as an array-like, reading elements up to fn.length
includes: [asyncHelpers.js, compareArray.js]
flags: [async]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
  const fn = function(a, b) {};
  fn[0] = 1;
  fn[1] = 2;
  fn[2] = 3;

  const result = await Array.fromAsync(fn);
  assert.compareArray(result, [1, 2]);
});
