// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.map
description: >
  The callbackfn returned values are applied to the new instance
info: |
  22.2.3.19 %TypedArray%.prototype.map ( callbackfn [ , thisArg ] )

  6. Let A be ? TypedArraySpeciesCreate(O, « len »).
  7. Let k be 0.
  8. Repeat, while k < len
    ...
    c. Let mappedValue be ? Call(callbackfn, T, « kValue, k, O »).
    d. Perform ? Set(A, Pk, mappedValue, true).
    ...
  9. Return A.
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([1n, 2n, 4n]));
  var result = sample.map(function(v) {
    return v * 3n;
  });

  assert.sameValue(result[0], 3n, "result[0] == 3");
  assert.sameValue(result[1], 6n, "result[1] == 6");
  assert.sameValue(result[2], 12n, "result[2] == 12");
});
