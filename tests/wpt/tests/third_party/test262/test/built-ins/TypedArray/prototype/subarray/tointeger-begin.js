// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.subarray
description: ToInteger(begin)
info: |
  22.2.3.27 %TypedArray%.prototype.subarray( begin , end )

  ...
  7. Let relativeBegin be ? ToInteger(begin).
  ...
includes: [testTypedArray.js, compareArray.js]
features: [TypedArray]
---*/

var obj = {
  valueOf: function() {
    return 2;
  }
};

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([40, 41, 42, 43]));

  assert(compareArray(sample.subarray(false), [40, 41, 42, 43]), "false");
  assert(compareArray(sample.subarray(true), [41, 42, 43]), "true");

  assert(compareArray(sample.subarray(NaN), [40, 41, 42, 43]), "NaN");
  assert(compareArray(sample.subarray(null), [40, 41, 42, 43]), "null");
  assert(compareArray(sample.subarray(undefined), [40, 41, 42, 43]), "undefined");

  assert(compareArray(sample.subarray(1.1), [41, 42, 43]), "1.1");
  assert(compareArray(sample.subarray(1.5), [41, 42, 43]), "1.5");
  assert(compareArray(sample.subarray(0.6), [40, 41, 42, 43]), "0.6");

  assert(compareArray(sample.subarray(-1.5), [43]), "-1.5");
  assert(compareArray(sample.subarray(-1.1), [43]), "-1.1");
  assert(compareArray(sample.subarray(-0.6), [40, 41, 42, 43]), "-0.6");

  assert(compareArray(sample.subarray("3"), [43]), "string");
  assert(
    compareArray(
      sample.subarray(obj),
      [42, 43]
    ),
    "object"
  );
});
