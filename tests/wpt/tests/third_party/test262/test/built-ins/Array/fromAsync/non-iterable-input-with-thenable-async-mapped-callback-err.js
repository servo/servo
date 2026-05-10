// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: Non-iterable input with thenable result promise rejects if async map function callback throws. 
includes: [asyncHelpers.js]
flags: [async]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
  const expectedValue = {};
  const inputThenable = {
    then (resolve, reject) {
      resolve(expectedValue);
    },
  };
  const input = {
    length: 1,
    0: inputThenable,
  };
  const outputPromise = Array.fromAsync(input, async v => {
    throw new Test262Error;
  });
  assert.throwsAsync(Test262Error, () => outputPromise);
});
