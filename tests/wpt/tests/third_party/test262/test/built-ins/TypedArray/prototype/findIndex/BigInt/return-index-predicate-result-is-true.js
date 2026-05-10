// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.findindex
description: >
  Return index if predicate return a boolean true value.
info: |
  22.2.3.11 %TypedArray%.prototype.findIndex ( predicate [ , thisArg ] )

  %TypedArray%.prototype.findIndex is a distinct function that implements the
  same algorithm as Array.prototype.findIndex as defined in 22.1.3.9 except that
  the this object's [[ArrayLength]] internal slot is accessed in place of
  performing a [[Get]] of "length".

  ...

  22.1.3.9 Array.prototype.findIndex ( predicate[ , thisArg ] )

  ...
  5. Let k be 0.
  6. Repeat, while k < len
    ...
    c. Let testResult be ToBoolean(? Call(predicate, T, « kValue, k, O »)).
    d. If testResult is true, return k.
  ...
includes: [testTypedArray.js]
features: [BigInt, Symbol, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([39n, 3n, 9n]));
  var called = 0;

  var result = sample.findIndex(function() {
    called++;
    return true;
  });

  assert.sameValue(result, 0, "returned true on sample[0]");
  assert.sameValue(called, 1, "predicate was called once");

  called = 0;
  result = sample.findIndex(function(val) {
    called++;
    return val === 9n;
  });

  assert.sameValue(called, 3, "predicate was called three times");
  assert.sameValue(result, 2, "returned true on sample[3]");

  result = sample.findIndex(function() { return "string"; });
  assert.sameValue(result, 0, "ToBoolean(string)");

  result = sample.findIndex(function() { return {}; });
  assert.sameValue(result, 0, "ToBoolean(object)");

  result = sample.findIndex(function() { return Symbol(""); });
  assert.sameValue(result, 0, "ToBoolean(symbol)");

  result = sample.findIndex(function() { return 1; });
  assert.sameValue(result, 0, "ToBoolean(number)");

  result = sample.findIndex(function() { return -1; });
  assert.sameValue(result, 0, "ToBoolean(negative number)");
});
