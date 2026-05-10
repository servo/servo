// Copyright (c) 2023 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-map.groupby
description: Map.groupBy throws when iterator next throws
info: |
  Map.groupBy ( items, callbackfn )

  ...

  GroupBy ( items, callbackfn, coercion )

  6. Repeat,
    b. Let next be ? IteratorStep(iteratorRecord).

  ...
features: [array-grouping, Map, Symbol.iterator]
---*/

const throwingIterator = {
  [Symbol.iterator]: function () {
    return this;
  },
  next: function next() {
    throw new Test262Error('next() method was called');
  }
};

assert.throws(Test262Error, function () {
  Map.groupBy(throwingIterator, function () {
    return 'key';
  });
});
