// Copyright (c) 2023 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-map.groupby
description: Map.groupBy populates Map with correct keys and values
info: |
  Map.groupBy ( items, callbackfn )
  ...
includes: [compareArray.js]
features: [array-grouping, Map]
---*/

const array = [1, 2, 3];

const map = Map.groupBy(array, function (i) {
  return i % 2 === 0 ? 'even' : 'odd';
});

assert.compareArray(Array.from(map.keys()), ['odd', 'even']);
assert.compareArray(map.get('even'), [2]);
assert.compareArray(map.get('odd'), [1, 3]);
