// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  Promise is rejected if the length of the array-like to copy is out of range
info: |
  j. If _iteratorRecord_ is not *undefined*, then
    ...
  k. Else,
    ...
    iv. If IsConstructor(_C_) is *true*, then
      ...
    v. Else,
      1. Let _A_ be ? ArrayCreate(_len_).

  ArrayCreate, step 1:
    1. If _length_ > 2³² - 1, throw a *RangeError* exception.
includes: [asyncHelpers.js]
flags: [async]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
  const notConstructor = {};

  await assert.throwsAsync(RangeError, () => Array.fromAsync.call(notConstructor, {
    length: 4294967296  // 2³²
  }), "Array-like with excessive length");
});
