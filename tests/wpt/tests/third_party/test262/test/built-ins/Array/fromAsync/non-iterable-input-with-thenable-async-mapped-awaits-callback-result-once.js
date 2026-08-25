// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: Non-iterable input with thenable result with async mapped awaits each callback result once.
includes: [asyncHelpers.js]
flags: [async]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
  let awaitCounter = 0;
  const input = {
    length: 3,
    0: 0,
    1: Promise.resolve(1),
    2: Promise.resolve(2),
    3: Promise.resolve(3), // This is ignored because the length is 3.
  };
  await Array.fromAsync(input, async v => {
    return {
      // This “then” method should occur three times:
      // one for each value from the input.
      then (resolve, reject) {
        awaitCounter ++;
        resolve(v);
      },
    };
  });
  assert.sameValue(awaitCounter, 3);
});
