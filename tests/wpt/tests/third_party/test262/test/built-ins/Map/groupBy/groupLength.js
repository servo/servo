// Copyright (c) 2023 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-map.groupby
description: Map.groupBy populates Map with correct keys and values
info: |
  Map.groupBy ( items, callbackfn )

  ...
includes: [compareArray.js]
features: [array-grouping, Map, Symbol.iterator]
---*/

const arr = ['hello', 'test', 'world'];

const map = Map.groupBy(arr, function (i) { return i.length; });

assert.compareArray(Array.from(map.keys()), [5, 4]);
assert.compareArray(map.get(5), ['hello', 'world']);
assert.compareArray(map.get(4), ['test']);
