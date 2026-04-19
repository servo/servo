// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.find
description: >
  Return undefined if predicate always returns a boolean false value.
info: |
  22.2.3.10 %TypedArray%.prototype.find (predicate [ , thisArg ] )

  %TypedArray%.prototype.find is a distinct function that implements the same
  algorithm as Array.prototype.find as defined in 22.1.3.8 except that the this
  object's [[ArrayLength]] internal slot is accessed in place of performing a
  [[Get]] of "length". The implementation of the algorithm may be optimized with
  the knowledge that the this value is an object that has a fixed length and
  whose integer indexed properties are not sparse.

  ...

  22.1.3.8 Array.prototype.find ( predicate[ , thisArg ] )

  ...
  6. Repeat, while k < len
    ...
    c. Let testResult be ToBoolean(? Call(predicate, T, « kValue, k, O »)).
    ...
  7. Return undefined.
includes: [testTypedArray.js]
features: [Symbol, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(3));
  var called = 0;

  var result = sample.find(function() {
    called++;
    return false;
  });

  assert.sameValue(called, 3, "predicate was called three times");
  assert.sameValue(result, undefined);

  result = sample.find(function() { return ""; });
  assert.sameValue(result, undefined, "ToBoolean(empty string)");

  result = sample.find(function() { return undefined; });
  assert.sameValue(result, undefined, "ToBoolean(undefined)");

  result = sample.find(function() { return null; });
  assert.sameValue(result, undefined, "ToBoolean(null)");

  result = sample.find(function() { return 0; });
  assert.sameValue(result, undefined, "ToBoolean(0)");

  result = sample.find(function() { return -0; });
  assert.sameValue(result, undefined, "ToBoolean(-0)");

  result = sample.find(function() { return NaN; });
  assert.sameValue(result, undefined, "ToBoolean(NaN)");
});
