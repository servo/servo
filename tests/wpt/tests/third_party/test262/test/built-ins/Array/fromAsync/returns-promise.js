// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: Array.fromAsync returns a Promise
info: |
  5. Return _promiseCapability_.[[Promise]].
flags: [async]
includes: [asyncHelpers.js]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
  let p = Array.fromAsync([0, 1, 2]);

  assert(p instanceof Promise, "Array.fromAsync returns a Promise");
  assert.sameValue(
    Object.getPrototypeOf(p),
    Promise.prototype,
    "Array.fromAsync returns an object with prototype Promise.prototype"
  );

  p = Array.fromAsync([0, 1, 2], () => {
    throw new Test262Error("this will make the Promise reject");
  })
  assert(p instanceof Promise, "Array.fromAsync returns a Promise even on error");
  assert.sameValue(
    Object.getPrototypeOf(p),
    Promise.prototype,
    "Array.fromAsync returns an object with prototype Promise.prototype even on error"
  );

  await assert.throwsAsync(Test262Error, () => p, "Prevent unhandled rejection");
});
