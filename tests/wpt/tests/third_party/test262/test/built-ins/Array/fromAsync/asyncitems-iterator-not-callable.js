// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  Array.fromAsync rejects if the @@iterator property is not callable
includes: [asyncHelpers.js]
flags: [async]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
  for (const v of [true, "", Symbol(), 1, 1n, {}]) {
    await assert.throwsAsync(TypeError,
      () => Array.fromAsync({ [Symbol.iterator]: v }),
      `@@iterator = ${typeof v}`);
  }
});
