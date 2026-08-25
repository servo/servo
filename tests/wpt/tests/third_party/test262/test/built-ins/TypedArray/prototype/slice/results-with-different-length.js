// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.slice
description: slice may return a new instance with a smaller length
includes: [testTypedArray.js, compareArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([40, 41, 42, 43]));

  function testRes(result, expected, msg) {
    assert(compareArray(result, expected), msg + ", result: [" + result + "]");
  }

  testRes(sample.slice(1), [41, 42, 43], "begin == 1");
  testRes(sample.slice(2), [42, 43], "begin == 2");
  testRes(sample.slice(3), [43], "begin == 3");

  testRes(sample.slice(1, 4), [41, 42, 43], "begin == 1, end == length");
  testRes(sample.slice(2, 4), [42, 43], "begin == 2, end == length");
  testRes(sample.slice(3, 4), [43], "begin == 3, end == length");

  testRes(sample.slice(0, 1), [40], "begin == 0, end == 1");
  testRes(sample.slice(0, 2), [40, 41], "begin == 0, end == 2");
  testRes(sample.slice(0, 3), [40, 41, 42], "begin == 0, end == 3");

  testRes(sample.slice(-1), [43], "begin == -1");
  testRes(sample.slice(-2), [42, 43], "begin == -2");
  testRes(sample.slice(-3), [41, 42, 43], "begin == -3");

  testRes(sample.slice(-1, 4), [43], "begin == -1, end == length");
  testRes(sample.slice(-2, 4), [42, 43], "begin == -2, end == length");
  testRes(sample.slice(-3, 4), [41, 42, 43], "begin == -3, end == length");

  testRes(sample.slice(0, -1), [40, 41, 42], "begin == 0, end == -1");
  testRes(sample.slice(0, -2), [40, 41], "begin == 0, end == -2");
  testRes(sample.slice(0, -3), [40], "begin == 0, end == -3");

  testRes(sample.slice(-0, -1), [40, 41, 42], "begin == -0, end == -1");
  testRes(sample.slice(-0, -2), [40, 41], "begin == -0, end == -2");
  testRes(sample.slice(-0, -3), [40], "begin == -0, end == -3");

  testRes(sample.slice(-2, -1), [42], "length == 4, begin == -2, end == -1");
  testRes(sample.slice(1, -1), [41, 42], "length == 4, begin == 1, end == -1");
  testRes(sample.slice(1, -2), [41], "length == 4, begin == 1, end == -2");
  testRes(sample.slice(2, -1), [42], "length == 4, begin == 2, end == -1");

  testRes(sample.slice(-1, 5), [43], "begin == -1, end > length");
  testRes(sample.slice(-2, 4), [42, 43], "begin == -2, end > length");
  testRes(sample.slice(-3, 4), [41, 42, 43], "begin == -3, end > length");
});
