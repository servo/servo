// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  Array.fromAsync tries the various properties in order and awaits promises
includes: [asyncHelpers.js, compareArray.js, temporalHelpers.js]
flags: [async]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
  const actual = [];
  const items = TemporalHelpers.propertyBagObserver(actual, {
    length: 2,
    0: Promise.resolve(2),
    1: Promise.resolve(1),
  }, "items");
  const result = await Array.fromAsync(items);
  assert.compareArray(result, [2, 1]);
  assert.compareArray(actual, [
    "get items[Symbol.asyncIterator]",
    "get items[Symbol.iterator]",
    "get items.length",
    "get items.length.valueOf",
    "call items.length.valueOf",
    "get items[0]",
    "get items[1]",
  ]);
});
