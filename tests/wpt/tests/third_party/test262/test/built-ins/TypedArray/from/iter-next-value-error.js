// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.from
description: Returns error produced by accessing iterated value
info: |
  22.2.2.1.1 Runtime Semantics: IterableToArrayLike( items )

  2. If usingIterator is not undefined, then
    ...
    d. Repeat, while next is not false
      ...
      ii. If next is not false, then
        1. Let nextValue be ? IteratorValue(next).
  ...
includes: [testTypedArray.js]
features: [Symbol.iterator, TypedArray]
---*/

var iter = {};
iter[Symbol.iterator] = function() {
  return {
    next: function() {
      var result = {};
      Object.defineProperty(result, 'value', {
        get: function() {
          throw new Test262Error();
        }
      });

      return result;
    }
  };
};

assert.throws(Test262Error, function() {
  TypedArray.from(iter);
});
