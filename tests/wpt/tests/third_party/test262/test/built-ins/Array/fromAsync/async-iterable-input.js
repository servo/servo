// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  Async-iterable input is transferred to the output array.
includes: [compareArray.js, asyncHelpers.js]
flags: [async]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
  const expected = [ 0, 1, 2 ];

  async function* generateInput () {
    yield* expected;
  }

  const input = generateInput();
  const output = await Array.fromAsync(input);
  assert.compareArray(output, expected);
});
