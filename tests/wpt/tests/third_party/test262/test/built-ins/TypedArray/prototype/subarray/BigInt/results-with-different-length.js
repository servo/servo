// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.subarray
description: Subarray may return a new instance with a smaller length
info: |
  22.2.3.27 %TypedArray%.prototype.subarray( begin , end )

  ...
  17. Return ? TypedArraySpeciesCreate(O, argumentsList).
includes: [testTypedArray.js, compareArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([40n, 41n, 42n, 43n]));

  function testRes(result, expected, msg) {
    assert(compareArray(result, expected), msg + ", result: [" + result + "]");
  }

  testRes(sample.subarray(1), [41n, 42n, 43n], "begin == 1");
  testRes(sample.subarray(2), [42n, 43n], "begin == 2");
  testRes(sample.subarray(3), [43n], "begin == 3");

  testRes(sample.subarray(1, 4), [41n, 42n, 43n], "begin == 1, end == length");
  testRes(sample.subarray(2, 4), [42n, 43n], "begin == 2, end == length");
  testRes(sample.subarray(3, 4), [43n], "begin == 3, end == length");

  testRes(sample.subarray(0, 1), [40n], "begin == 0, end == 1");
  testRes(sample.subarray(0, 2), [40n, 41n], "begin == 0, end == 2");
  testRes(sample.subarray(0, 3), [40n, 41n, 42n], "begin == 0, end == 3");

  testRes(sample.subarray(-1), [43n], "begin == -1");
  testRes(sample.subarray(-2), [42n, 43n], "begin == -2");
  testRes(sample.subarray(-3), [41n, 42n, 43n], "begin == -3");

  testRes(sample.subarray(-1, 4), [43n], "begin == -1, end == length");
  testRes(sample.subarray(-2, 4), [42n, 43n], "begin == -2, end == length");
  testRes(sample.subarray(-3, 4), [41n, 42n, 43n], "begin == -3, end == length");

  testRes(sample.subarray(0, -1), [40n, 41n, 42n], "begin == 0, end == -1");
  testRes(sample.subarray(0, -2), [40n, 41n], "begin == 0, end == -2");
  testRes(sample.subarray(0, -3), [40n], "begin == 0, end == -3");

  testRes(sample.subarray(-0, -1), [40n, 41n, 42n], "begin == -0, end == -1");
  testRes(sample.subarray(-0, -2), [40n, 41n], "begin == -0, end == -2");
  testRes(sample.subarray(-0, -3), [40n], "begin == -0, end == -3");

  testRes(sample.subarray(-2, -1), [42n], "length == 4, begin == -2, end == -1");
  testRes(sample.subarray(1, -1), [41n, 42n], "length == 4, begin == 1, end == -1");
  testRes(sample.subarray(1, -2), [41n], "length == 4, begin == 1, end == -2");
  testRes(sample.subarray(2, -1), [42n], "length == 4, begin == 2, end == -1");

  testRes(sample.subarray(-1, 5), [43n], "begin == -1, end > length");
  testRes(sample.subarray(-2, 4), [42n, 43n], "begin == -2, end > length");
  testRes(sample.subarray(-3, 4), [41n, 42n, 43n], "begin == -3, end > length");
});
