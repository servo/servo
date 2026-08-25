// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  Constructs an intrinsic Array if this-value is not a constructor, and length
  and element properties are set accordingly.
info: |
  3.e. If IsConstructor(_C_) is *true*, then
    ...
  f. Else,
    i. Let _A_ be ! ArrayCreate(0).

  ...
  j. If _iteratorRecord_ is not *undefined*, then
    ...
  k. Else,
    ...
    iv. If IsConstructor(_C_) is *true*, then
      ...
    v. Else,
      1. Let _A_ be ? ArrayCreate(_len_).
includes: [compareArray.js, asyncHelpers.js]
flags: [async]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
  const thisValue = {
    length: 4000,
    0: 32,
    1: 64,
    2: 128
  };

  let result = await Array.fromAsync.call(thisValue, [1, 2]);
  assert(Array.isArray(result), "result is an intrinsic Array");
  assert.compareArray(result, [1, 2], "result is not disrupted by properties of this-value");

  result = await Array.fromAsync.call(thisValue, {
    length: 2,
    0: 1,
    1: 2
  });
  assert(Array.isArray(result), "result is an intrinsic Array");
  assert.compareArray(result, [1, 2], "result is not disrupted by properties of this-value");
});
