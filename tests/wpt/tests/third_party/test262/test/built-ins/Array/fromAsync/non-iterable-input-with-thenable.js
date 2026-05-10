// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  Non iterable input with thenables is transferred to the output array.
includes: [compareArray.js, asyncHelpers.js]
flags: [async]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
  const expected = [ 0, 1, 2 ];
  const input = {
    length: 3,
    0: 0,
    1: Promise.resolve(1),
    2: Promise.resolve(2),
    3: Promise.resolve(3), // This is ignored because the length is 3.
  };
  const output = await Array.fromAsync(input);
  assert.compareArray(output, expected);
});
