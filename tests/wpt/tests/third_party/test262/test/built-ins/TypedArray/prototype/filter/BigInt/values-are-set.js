// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.filter
description: >
  Returned instance with filtered values set on it
info: |
  22.2.3.9 %TypedArray%.prototype.filter ( callbackfn [ , thisArg ] )

  ...
  12. For each element e of kept
    a. Perform ! Set(A, ! ToString(n), e, true).
    b. Increment n by 1.
  13. Return A.
includes: [testTypedArray.js, compareArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([41n, 1n, 42n, 7n]));
  var result;

  result = sample.filter(function() { return true; });
  assert(compareArray(result, [41n, 1n, 42n, 7n]), "values are set #1");

  result = sample.filter(function(v) {
    return v > 40n;
  });
  assert(compareArray(result, [41n, 42n]), "values are set #2");
});
