// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: Async-iterable input does not await input values.
includes: [compareArray.js, asyncHelpers.js]
flags: [async]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
  const prom = Promise.resolve({});
  const expected = [ prom ];

  function createInput () {
    return {
      // The following async iterator will yield one value
      // (the promise named “prom”).
      [Symbol.asyncIterator]() {
        let i = 0;
        return {
          async next() {
            if (i > 0) {
              return { done: true };
            }
            i++;
            return { value: prom, done: false }
          },
        };
      },
    };
  }

  const input = createInput();
  const output = await Array.fromAsync(input);
  assert.compareArray(output, expected);
});
