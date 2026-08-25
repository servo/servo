// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: Sync-iterable input with thenable result promise rejects if async map function callback throws. 
includes: [asyncHelpers.js]
flags: [async]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
  const input = [ Promise.resolve(0) ].values();
  const outputPromise = Array.fromAsync(input, async v => {
    throw new Test262Error;
  });
  await assert.throwsAsync(Test262Error, () => outputPromise);
});
