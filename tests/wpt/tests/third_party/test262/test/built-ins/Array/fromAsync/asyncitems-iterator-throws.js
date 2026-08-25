// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  Array.fromAsync rejects if getting the @@iterator property throws
includes: [asyncHelpers.js]
flags: [async]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
  await assert.throwsAsync(Test262Error,
    () => Array.fromAsync({ get [Symbol.iterator]() { throw new Test262Error() } }));
});
