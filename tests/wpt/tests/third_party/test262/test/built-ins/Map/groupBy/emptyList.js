// Copyright (c) 2023 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-map.groupby
description: Callback is not called and object is not populated if the iterable is empty
info: |
  Map.groupBy ( items, callbackfn )

  ...
  GroupBy ( items, callbackfn, coercion )

  6. Repeat,
    c. If next is false, then
      i. Return groups.
  ...
features: [array-grouping, Map]
---*/

const original = [];

const map = Map.groupBy(original, function () {
  throw new Test262Error('callback function should not be called')
});

assert.notSameValue(original, map, 'Map.groupBy returns a map');
assert.sameValue(map.size, 0);
