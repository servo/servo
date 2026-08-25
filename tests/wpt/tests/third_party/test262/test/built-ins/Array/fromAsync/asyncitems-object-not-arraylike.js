// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  Treats an asyncItems object that isn't an array-like as a 0-length array-like
info: |
  3.k.iii. Let _len_ be ? LengthOfArrayLike(_arrayLike_).
features: [Array.fromAsync]
flags: [async]
includes: [asyncHelpers.js, compareArray.js]
---*/

asyncTest(async function () {
  const notArrayLike = Object.create(null);
  notArrayLike[0] = 0;
  notArrayLike[1] = 1;
  notArrayLike[2] = 2;

  const array = await Array.fromAsync(notArrayLike);
  assert.compareArray(array, [], "non-array-like object is treated as 0-length array-like");
});
