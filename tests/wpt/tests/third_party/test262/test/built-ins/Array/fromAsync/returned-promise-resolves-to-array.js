// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  Array.fromAsync returns a Promise that resolves to an Array in the normal case
info: |
  1. Let _C_ be the *this* value.
  ...
  3.e. If IsConstructor(_C_) is *true*, then
    i. Let _A_ be ? Construct(_C_).
features: [Array.fromAsync]
flags: [async]
includes: [asyncHelpers.js]
---*/

asyncTest(async function () {
  const promise = Array.fromAsync([0, 1, 2]);
  const array = await promise;
  assert(Array.isArray(array), "Array.fromAsync returns a Promise that resolves to an Array");
});
