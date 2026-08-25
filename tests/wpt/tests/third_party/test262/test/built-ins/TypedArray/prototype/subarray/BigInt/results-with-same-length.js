// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.subarray
description: Subarray may return a new instance with the same length
info: |
  22.2.3.27 %TypedArray%.prototype.subarray( begin , end )

  ...
  17. Return ? TypedArraySpeciesCreate(O, argumentsList).
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([40n, 41n, 42n, 43n]));

  function testRes(result, msg) {
    assert.sameValue(result.length, 4, msg);
    assert.sameValue(result[0], 40n, msg + " & result[0] === 40");
    assert.sameValue(result[1], 41n, msg + " & result[1] === 41");
    assert.sameValue(result[2], 42n, msg + " & result[2] === 42");
    assert.sameValue(result[3], 43n, msg + " & result[3] === 43");
  }

  testRes(sample.subarray(0), "begin == 0");
  testRes(sample.subarray(-4), "begin == -srcLength");
  testRes(sample.subarray(-5), "begin < -srcLength");

  testRes(sample.subarray(0, 4), "begin == 0, end == srcLength");
  testRes(sample.subarray(-4, 4), "begin == -srcLength, end == srcLength");
  testRes(sample.subarray(-5, 4), "begin < -srcLength, end == srcLength");

  testRes(sample.subarray(0, 5), "begin == 0, end > srcLength");
  testRes(sample.subarray(-4, 5), "begin == -srcLength, end > srcLength");
  testRes(sample.subarray(-5, 5), "begin < -srcLength, end > srcLength");
});
