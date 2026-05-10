// Copyright (c) 2023 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.groupby
description: Object.groupBy coerces return value with ToPropertyKey
info: |
  Object.groupBy ( items, callbackfn )

  ...

  GroupBy ( items, callbackfn, coercion )

  6. Repeat,
    g. If coercion is property, then
      i. Set key to Completion(ToPropertyKey(key)).
      ii. IfAbruptCloseIterator(key, iteratorRecord).

  ...
includes: [compareArray.js]
features: [array-grouping]
---*/

let calls = 0;
const stringable = {
  toString: function toString() {
    return 1;
  }
}

const array = [1, '1', stringable];

const obj = Object.groupBy(array, function (v) { return v; });

assert.compareArray(Object.keys(obj), ['1']);
assert.compareArray(obj['1'], [1, '1', stringable]);
