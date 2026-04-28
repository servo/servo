// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.slice
description: slice may return a new empty instance
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([40n, 41n, 42n, 43n]));

  function testRes(result, msg) {
    assert.sameValue(result.length, 0, msg);
    assert.sameValue(
      result.hasOwnProperty(0),
      false,
      msg + " & result.hasOwnProperty(0) === false"
    );
  }

  testRes(sample.slice(4), "begin == length");
  testRes(sample.slice(5), "begin > length");

  testRes(sample.slice(4, 4), "begin == length, end == length");
  testRes(sample.slice(5, 4), "begin > length, end == length");

  testRes(sample.slice(4, 5), "begin == length, end > length");
  testRes(sample.slice(5, 5), "begin > length, end > length");

  testRes(sample.slice(0, 0), "begin == 0, end == 0");
  testRes(sample.slice(-0, -0), "begin == -0, end == -0");
  testRes(sample.slice(1, 0), "begin > 0, end == 0");
  testRes(sample.slice(-1, 0), "being < 0, end == 0");

  testRes(sample.slice(2, 1), "begin > 0, begin < length, begin > end, end > 0");
  testRes(sample.slice(2, 2), "begin > 0, begin < length, begin == end");

  testRes(sample.slice(2, -2), "begin > 0, begin < length, end == -2");

  testRes(sample.slice(-1, -1), "length = 4, begin == -1, end == -1");
  testRes(sample.slice(-1, -2), "length = 4, begin == -1, end == -2");
  testRes(sample.slice(-2, -2), "length = 4, begin == -2, end == -2");

  testRes(sample.slice(0, -4), "begin == 0, end == -length");
  testRes(sample.slice(-4, -4), "begin == -length, end == -length");
  testRes(sample.slice(-5, -4), "begin < -length, end == -length");

  testRes(sample.slice(0, -5), "begin == 0, end < -length");
  testRes(sample.slice(-4, -5), "begin == -length, end < -length");
  testRes(sample.slice(-5, -5), "begin < -length, end < -length");
});
