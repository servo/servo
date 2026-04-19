// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  Use the intrinsic @@iterator and @@asyncIterator to check iterability
includes: [compareArray.js, asyncHelpers.js]
flags: [async]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
  // Replace the user-reachable Symbol.iterator and Symbol.asyncIterator with
  // fake symbol keys
  const originalSymbol = globalThis.Symbol;
  const fakeIteratorSymbol = Symbol("iterator");
  const fakeAsyncIteratorSymbol = Symbol("asyncIterator");
  globalThis.Symbol = {
    iterator: fakeIteratorSymbol,
    asyncIterator: fakeAsyncIteratorSymbol,
  };

  const input = {
    length: 3,
    0: 0,
    1: 1,
    2: 2,
    [fakeIteratorSymbol]() {
      throw new Test262Error("The fake Symbol.iterator method should not be called");
    },
    [fakeAsyncIteratorSymbol]() {
      throw new Test262Error("The fake Symbol.asyncIterator method should not be called");
    }
  };
  const output = await Array.fromAsync(input);
  assert.compareArray(output, [0, 1, 2]);

  globalThis.Symbol = originalSymbol;
});
