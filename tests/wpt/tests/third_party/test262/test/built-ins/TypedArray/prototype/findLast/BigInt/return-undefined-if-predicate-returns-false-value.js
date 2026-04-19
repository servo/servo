// Copyright (C) 2021 Microsoft. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.findlast
description: >
  Return undefined if predicate always returns a boolean false value.
info: |
  %TypedArray%.prototype.findLast (predicate [ , thisArg ] )

  ...
  6. 6. Repeat, while k ≥ 0
    ...
    c. Let testResult be ! ToBoolean(? Call(predicate, thisArg, « kValue, 𝔽(k), O »)).
    ...
  7. Return undefined.
includes: [testTypedArray.js]
features: [BigInt, Symbol, TypedArray, array-find-from-last]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(3));
  var called = 0;

  var result = sample.findLast(function() {
    called++;
    return false;
  });

  assert.sameValue(called, 3, "predicate was called three times");
  assert.sameValue(result, undefined);

  result = sample.findLast(function() { return ""; });
  assert.sameValue(result, undefined, "ToBoolean(empty string)");

  result = sample.findLast(function() { return undefined; });
  assert.sameValue(result, undefined, "ToBoolean(undefined)");

  result = sample.findLast(function() { return null; });
  assert.sameValue(result, undefined, "ToBoolean(null)");

  result = sample.findLast(function() { return 0; });
  assert.sameValue(result, undefined, "ToBoolean(0)");

  result = sample.findLast(function() { return -0; });
  assert.sameValue(result, undefined, "ToBoolean(-0)");

  result = sample.findLast(function() { return NaN; });
  assert.sameValue(result, undefined, "ToBoolean(NaN)");
});
