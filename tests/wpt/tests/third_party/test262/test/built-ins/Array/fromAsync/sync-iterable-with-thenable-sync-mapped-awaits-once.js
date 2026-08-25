// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  Sync-iterable input with mapfn awaits each input once with sync mapping callback
includes: [asyncHelpers.js]
flags: [async]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
  const expectedValue = {};
  let awaitCounter = 0;
  const inputThenable = {
    then (resolve, reject) {
      awaitCounter++;
      resolve(expectedValue);
    },
  };
  const input = [ inputThenable ].values();
  await Array.fromAsync(input, v => v);
  assert.sameValue(awaitCounter, 1);
});
