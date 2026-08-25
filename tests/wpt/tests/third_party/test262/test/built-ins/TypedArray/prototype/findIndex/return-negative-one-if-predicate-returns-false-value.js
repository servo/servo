// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.findindex
description: >
  Return -1 if predicate always returns a boolean false value.
info: |
  22.2.3.11 %TypedArray%.prototype.findIndex ( predicate [ , thisArg ] )

  %TypedArray%.prototype.findIndex is a distinct function that implements the
  same algorithm as Array.prototype.findIndex as defined in 22.1.3.9 except that
  the this object's [[ArrayLength]] internal slot is accessed in place of
  performing a [[Get]] of "length".

  ...

  22.1.3.9 Array.prototype.findIndex ( predicate[ , thisArg ] )

  ...
  6. Repeat, while k < len
    ...
    c. Let testResult be ToBoolean(? Call(predicate, T, « kValue, k, O »)).
  ...
  7. Return -1.
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([1, 2, 3]));
  var called = 0;

  var result = sample.findIndex(function() {
    called++;
    return false;
  });

  assert.sameValue(called, 3, "predicate was called three times");
  assert.sameValue(result, -1, "result is -1 when predicate returns are false");

  result = sample.findIndex(function() { return ""; });
  assert.sameValue(result, -1, "ToBoolean(string)");

  result = sample.findIndex(function() { return undefined; });
  assert.sameValue(result, -1, "ToBoolean(undefined)");

  result = sample.findIndex(function() { return null; });
  assert.sameValue(result, -1, "ToBoolean(null)");

  result = sample.findIndex(function() { return 0; });
  assert.sameValue(result, -1, "ToBoolean(0)");

  result = sample.findIndex(function() { return -0; });
  assert.sameValue(result, -1, "ToBoolean(-0)");

  result = sample.findIndex(function() { return NaN; });
  assert.sameValue(result, -1, "ToBoolean(NaN)");
});
