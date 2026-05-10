// Copyright (c) 2023 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-map.groupby
description: Map.groupBy returns a Map instance
info: |
  Map.groupBy ( items, callbackfn )

  ...

  2. Let map be ! Construct(%Map%).
  ...
  4. Return map.

  ...
features: [array-grouping, Map]
---*/

const array = [1, 2, 3];

const map = Map.groupBy(array, function (i) {
  return i % 2 === 0 ? 'even' : 'odd';
});

assert.sameValue(map instanceof Map, true);
