// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  Array.fromAsync doesn't special-case ArrayBuffer
includes: [asyncHelpers.js, compareArray.js]
flags: [async]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
  const items = new ArrayBuffer(7);
  const result = await Array.fromAsync(items);
  assert.compareArray(result, []);
});
