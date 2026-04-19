// Copyright (c) 2023 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.groupby
description: Object.groupBy throws when callback throws
info: |
  Object.groupBy ( items, callbackfn )

  ...
  GroupBy ( items, callbackfn, coercion )

  6. Repeat,
    e. Let key be Completion(Call(callbackfn, undefined, « value, 𝔽(k) »)).
    f. IfAbruptCloseIterator(key, iteratorRecord).
  ...
features: [array-grouping]
---*/

assert.throws(Test262Error, function() {
  const array = [1];
  Object.groupBy(array, function() {
    throw new Test262Error('throw in callback');
  })
});
