// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.findlastindex
description: >
  Return index if predicate return a boolean true value.
info: |
  %TypedArray%.prototype.findLastIndex ( predicate [ , thisArg ] )

  ...
  5. Let k be len - 1.
  6. Repeat, while k ≥ 0
    ...
    c. Let testResult be ! ToBoolean(? Call(predicate, thisArg, « kValue, 𝔽(k), O »)).
    d. If testResult is true, return 𝔽(k).
  ...
includes: [testTypedArray.js]
features: [BigInt, Symbol, TypedArray, array-find-from-last]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([39n, 3n, 9n]));
  var called = 0;

  var result = sample.findLastIndex(function() {
    called++;
    return true;
  });

  assert.sameValue(result, 2, "returned true on sample[2]");
  assert.sameValue(called, 1, "predicate was called once");

  called = 0;
  result = sample.findLastIndex(function(val) {
    called++;
    return val === 39n;
  });

  assert.sameValue(called, 3, "predicate was called three times");
  assert.sameValue(result, 0, "returned true on sample[0]");

  result = sample.findLastIndex(function() { return "string"; });
  assert.sameValue(result, 2, "ToBoolean(string)");

  result = sample.findLastIndex(function() { return {}; });
  assert.sameValue(result, 2, "ToBoolean(object)");

  result = sample.findLastIndex(function() { return Symbol(""); });
  assert.sameValue(result, 2, "ToBoolean(symbol)");

  result = sample.findLastIndex(function() { return 1; });
  assert.sameValue(result, 2, "ToBoolean(number)");

  result = sample.findLastIndex(function() { return -1; });
  assert.sameValue(result, 2, "ToBoolean(negative number)");
});
