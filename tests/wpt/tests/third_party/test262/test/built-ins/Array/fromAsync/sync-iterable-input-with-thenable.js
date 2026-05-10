// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: Sync-iterable input with thenables is transferred to the output array.
includes: [compareArray.js, asyncHelpers.js]
flags: [async]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
  const expected = [ 0, 1, 2 ];
  const input = [  0, Promise.resolve(1), Promise.resolve(2) ].values();
  const output = await Array.fromAsync(input);
  assert.compareArray(output, expected);
});
