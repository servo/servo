// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.filter
description: >
  Returns empty if every callbackfn returns boolean false
info: |
  22.2.3.9 %TypedArray%.prototype.filter ( callbackfn [ , thisArg ] )

  ...
  12. For each element e of kept
    a. Perform ! Set(A, ! ToString(n), e, true).
    b. Increment n by 1.
  13. Return A.
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(3));

  [
    false,
    "",
    0,
    -0,
    NaN,
    undefined,
    null
  ].forEach(function(val) {
    var result = sample.filter(function() {
      return val;
    });
    assert.sameValue(result.length, 0, val);
  });
});
