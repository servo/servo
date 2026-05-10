// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.from
description: Returns error produced by invoking @@iterator
info: |
  22.2.2.1 %TypedArray%.from ( source [ , mapfn [ , thisArg ] ] )

  ...
  6. Let arrayLike be ? IterableToArrayLike(source).
  ...

  22.2.2.1.1 Runtime Semantics: IterableToArrayLike( items )

  1. Let usingIterator be ? GetMethod(items, @@iterator).
  2. If usingIterator is not undefined, then
    a. Let iterator be ? GetIterator(items, usingIterator).
  ...
includes: [testTypedArray.js]
features: [Symbol.iterator, TypedArray]
---*/

var iter = {};
iter[Symbol.iterator] = function() {
  throw new Test262Error();
};

assert.throws(Test262Error, function() {
  TypedArray.from(iter);
});
