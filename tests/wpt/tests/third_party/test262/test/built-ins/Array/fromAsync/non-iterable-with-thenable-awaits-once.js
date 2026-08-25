// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: Non-iterable with thenables awaits each input value once without mapping callback.
includes: [asyncHelpers.js]
flags: [async]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
  const expectedValue = {};
  let awaitCounter = 0;
  const inputThenable = {
    then (resolve, reject) {
      awaitCounter ++;
      resolve(expectedValue);
    },
  };
  const input = {
    length: 1,
    0: inputThenable,
  };
 await Array.fromAsync(input);
 assert.sameValue(awaitCounter, 1);
});
