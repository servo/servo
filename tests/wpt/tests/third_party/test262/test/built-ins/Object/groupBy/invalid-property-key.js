// Copyright (c) 2023 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.groupby
description: Object.groupBy errors when callback return value cannot be converted to a property key.
info: |
  Object.groupBy ( items, callbackfn )

  ...
  GroupBy ( items, callbackfn, coercion )

  6. Repeat,
    g. If coercion is property, then
      i. Set key to Completion(ToPropertyKey(key)).
      ii. IfAbruptCloseIterator(key, iteratorRecord).

  ...
features: [array-grouping]
---*/

assert.throws(Test262Error, function () {
  const array = [1];
  Object.groupBy(array, function () {
    return {
      toString() {
        throw new Test262Error('not a property key');
      }
    };
  })
});
