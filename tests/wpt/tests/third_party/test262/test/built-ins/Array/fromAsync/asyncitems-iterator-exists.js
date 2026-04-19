// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.fromasync
description: >
  Array.fromAsync handles a sync iterator returned from @@iterator
includes: [asyncHelpers.js, compareArray.js, temporalHelpers.js]
flags: [async]
features: [Array.fromAsync]
---*/

asyncTest(async function () {
  function * syncGen() {
    for (let i = 0; i < 4; i++) {
      yield i * 2;
    }
  }

  const actual = [];
  const items = {};
  TemporalHelpers.observeProperty(actual, items, Symbol.asyncIterator, undefined, "items");
  TemporalHelpers.observeProperty(actual, items, Symbol.iterator, syncGen, "items");
  TemporalHelpers.observeProperty(actual, items, "length", 2, "items");
  TemporalHelpers.observeProperty(actual, items, 0, 2, "items");
  TemporalHelpers.observeProperty(actual, items, 1, 1, "items");
  const result = await Array.fromAsync(items);
  assert.compareArray(result, [0, 2, 4, 6]);
  assert.compareArray(actual, [
    "get items[Symbol.asyncIterator]",
    "get items[Symbol.iterator]",
  ]);
});
