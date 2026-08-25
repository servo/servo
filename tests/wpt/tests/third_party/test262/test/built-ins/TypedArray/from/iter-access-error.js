// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.from
description: Returns error produced by accessing @@iterator
info: |
  22.2.2.1 %TypedArray%.from ( source [ , mapfn [ , thisArg ] ] )

  ...
  6. Let arrayLike be ? IterableToArrayLike(source).
  ...

  22.2.2.1.1 Runtime Semantics: IterableToArrayLike( items )

  1. Let usingIterator be ? GetMethod(items, @@iterator).
  ...
includes: [testTypedArray.js]
features: [Symbol.iterator, TypedArray]
---*/

var iter = {};
Object.defineProperty(iter, Symbol.iterator, {
  get: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  TypedArray.from(iter);
});
