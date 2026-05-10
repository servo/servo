// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: Array-like object with holes treats the holes as undefined
info: |
  3.k.vii.2. Let _kValue_ be ? Get(_arrayLike_, _Pk_).
features: [Array.fromAsync]
flags: [async]
includes: [asyncHelpers.js, compareArray.js]
---*/

asyncTest(async function () {
  const arrayLike = Object.create(null);
  arrayLike.length = 5;
  arrayLike[0] = 0;
  arrayLike[1] = 1;
  arrayLike[2] = 2;
  arrayLike[4] = 4;

  const array = await Array.fromAsync(arrayLike);
  assert.compareArray(array, [0, 1, 2, undefined, 4], "holes in array-like treated as undefined");
});
