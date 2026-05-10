// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.filter
description: >
  Returns full length result if every callbackfn returns boolean false
info: |
  22.2.3.9 %TypedArray%.prototype.filter ( callbackfn [ , thisArg ] )

  ...
  12. For each element e of kept
    a. Perform ! Set(A, ! ToString(n), e, true).
    b. Increment n by 1.
  13. Return A.
includes: [testTypedArray.js, compareArray.js]
features: [BigInt, Symbol, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([40n, 41n, 42n]));

  [
    true,
    1,
    "test262",
    Symbol("1"),
    {},
    [],
    -1,
    Infinity,
    -Infinity,
    0.1,
    -0.1
  ].forEach(function(val) {
    var result = sample.filter(function() { return val; });
    assert(compareArray(result, sample), val);
  });
});
