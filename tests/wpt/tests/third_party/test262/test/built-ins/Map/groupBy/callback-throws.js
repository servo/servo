// Copyright (c) 2023 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-map.groupby
description: Map.groupBy throws when callback throws
info: |
  Map.groupBy ( items, callbackfn )

  ...
  GroupBy ( items, callbackfn, coercion )

  6. Repeat,
    e. Let key be Completion(Call(callbackfn, undefined, « value, 𝔽(k) »)).
    f. IfAbruptCloseIterator(key, iteratorRecord).
  ...
features: [array-grouping, Map]
---*/

assert.throws(Test262Error, function() {
  const array = [1];
  Map.groupBy(array, function() {
    throw new Test262Error('throw in callback');
  })
});
