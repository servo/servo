// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.findlast
description: >
  Return found value if predicate return a boolean true value.
info: |
  %TypedArray%.prototype.findLast (predicate [ , thisArg ] )

  6. Repeat, while k ≥ 0,
    ...
    c. Let testResult be ! ToBoolean(? Call(predicate, thisArg, « kValue, 𝔽(k), O »)).
    d. If testResult is true, return kValue.
    ...
includes: [testTypedArray.js]
features: [Symbol, TypedArray, array-find-from-last]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([39, 2, 62]));
  var called, result;

  called = 0;
  result = sample.findLast(function() {
    called++;
    return true;
  });
  assert.sameValue(result, 62, "returned true on sample[2]");
  assert.sameValue(called, 1, "predicate was called once");

  called = 0;
  result = sample.findLast(function(val) {
    called++;
    return val === 39;
  });
  assert.sameValue(called, 3, "predicate was called three times");
  assert.sameValue(result, 39, "returned true on sample[0]");

  result = sample.findLast(function() { return "string"; });
  assert.sameValue(result, 62, "ToBoolean(string)");

  result = sample.findLast(function() { return {}; });
  assert.sameValue(result, 62, "ToBoolean(object)");

  result = sample.findLast(function() { return Symbol(""); });
  assert.sameValue(result, 62, "ToBoolean(symbol)");

  result = sample.findLast(function() { return 1; });
  assert.sameValue(result, 62, "ToBoolean(number)");

  result = sample.findLast(function() { return -1; });
  assert.sameValue(result, 62, "ToBoolean(negative number)");
});
