// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.findlastindex
description: >
  Return -1 if predicate always returns a boolean false value.
info: |
  %TypedArray%.prototype.findLastIndex ( predicate [ , thisArg ] )

  ...
  5. Let k be len - 1.
  6. Repeat, while k ≥ 0
    ...
    c. Let testResult be ! ToBoolean(? Call(predicate, thisArg, « kValue, 𝔽(k), O »)).
    ...
  7. Return -1𝔽.
includes: [testTypedArray.js]
features: [BigInt, TypedArray, array-find-from-last]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([1n, 2n, 3n]));
  var called = 0;

  var result = sample.findLastIndex(function() {
    called++;
    return false;
  });

  assert.sameValue(called, 3, "predicate was called three times");
  assert.sameValue(result, -1, "result is -1 when predicate returns are false");

  result = sample.findLastIndex(function() { return ""; });
  assert.sameValue(result, -1, "ToBoolean(string)");

  result = sample.findLastIndex(function() { return undefined; });
  assert.sameValue(result, -1, "ToBoolean(undefined)");

  result = sample.findLastIndex(function() { return null; });
  assert.sameValue(result, -1, "ToBoolean(null)");

  result = sample.findLastIndex(function() { return 0; });
  assert.sameValue(result, -1, "ToBoolean(0)");

  result = sample.findLastIndex(function() { return -0; });
  assert.sameValue(result, -1, "ToBoolean(-0)");

  result = sample.findLastIndex(function() { return NaN; });
  assert.sameValue(result, -1, "ToBoolean(NaN)");
});
