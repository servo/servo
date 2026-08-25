// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.slice
description: slice may return a new instance with the same length
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([40, 41, 42, 43]));

  function testRes(result, msg) {
    assert.sameValue(result.length, 4, msg);
    assert.sameValue(result[0], 40, msg + " & result[0] === 40");
    assert.sameValue(result[1], 41, msg + " & result[1] === 41");
    assert.sameValue(result[2], 42, msg + " & result[2] === 42");
    assert.sameValue(result[3], 43, msg + " & result[3] === 43");
  }

  testRes(sample.slice(0), "begin == 0");
  testRes(sample.slice(-4), "begin == -srcLength");
  testRes(sample.slice(-5), "begin < -srcLength");

  testRes(sample.slice(0, 4), "begin == 0, end == srcLength");
  testRes(sample.slice(-4, 4), "begin == -srcLength, end == srcLength");
  testRes(sample.slice(-5, 4), "begin < -srcLength, end == srcLength");

  testRes(sample.slice(0, 5), "begin == 0, end > srcLength");
  testRes(sample.slice(-4, 5), "begin == -srcLength, end > srcLength");
  testRes(sample.slice(-5, 5), "begin < -srcLength, end > srcLength");
});
