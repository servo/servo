// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  Array.fromAsync promise rejects if iteration of input fails.
flags: [async]
features: [Array.fromAsync]
includes: [asyncHelpers.js]
---*/

asyncTest(async function () {
  async function *generateInput () {
    throw new Test262Error('This error should be propagated.');
  }
  const input = generateInput();
  const outputPromise = Array.fromAsync(input);
  await assert.throwsAsync(Test262Error, () => outputPromise);
});
