// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: Rejects on array-like object whose length cannot be gotten
info: |
  3.k.iii. Let _len_ be ? LengthOfArrayLike(_arrayLike_).
features: [Array.fromAsync]
flags: [async]
includes: [asyncHelpers.js]
---*/

asyncTest(async function () {
  await assert.throwsAsync(Test262Error, () => Array.fromAsync({
    get length() {
      throw new Test262Error('accessing length property fails');
    }
  }), "Promise should be rejected if array-like length getter throws");

  await assert.throwsAsync(TypeError, () => Array.fromAsync({
    length: 1n,
    0: 0
  }), "Promise should be rejected if array-like length can't be converted to a number");
});
