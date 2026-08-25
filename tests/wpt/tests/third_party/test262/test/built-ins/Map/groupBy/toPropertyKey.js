// Copyright (c) 2023 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-map.groupby
description: Map.groupBy does not coerce return value with ToPropertyKey
info: |
  Map.groupBy ( items, callbackfn )

  ...
includes: [compareArray.js]
features: [array-grouping, Map]
---*/

let calls = 0;
const stringable = {
  toString: function toString() {
    return 1;
  }
};

const array = [1, '1', stringable];

const map = Map.groupBy(array, function (v) { return v; });

assert.compareArray(Array.from(map.keys()), [1, '1', stringable]);
assert.compareArray(map.get('1'), ['1']);
assert.compareArray(map.get(1), [1]);
assert.compareArray(map.get(stringable), [stringable]);
