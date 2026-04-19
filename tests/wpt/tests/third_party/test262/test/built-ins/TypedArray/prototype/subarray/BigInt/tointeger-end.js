// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.subarray
description: ToInteger(end)
info: |
  22.2.3.27 %TypedArray%.prototype.subarray( begin , end )

  ...
  9. If end is undefined, let relativeEnd be srcLength; else, let relativeEnd be
  ? ToInteger(end).
  ...
includes: [testTypedArray.js, compareArray.js]
features: [BigInt, TypedArray]
---*/

var obj = {
  valueOf: function() {
    return 2;
  }
};

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([40n, 41n, 42n, 43n]));

  assert(compareArray(sample.subarray(0, false), []), "false");
  assert(compareArray(sample.subarray(0, true), [40n]), "true");

  assert(compareArray(sample.subarray(0, NaN), []), "NaN");
  assert(compareArray(sample.subarray(0, null), []), "null");
  assert(compareArray(sample.subarray(0, undefined), [40n, 41n, 42n, 43n]), "undefined");

  assert(compareArray(sample.subarray(0, 0.6), []), "0.6");
  assert(compareArray(sample.subarray(0, 1.1), [40n]), "1.1");
  assert(compareArray(sample.subarray(0, 1.5), [40n]), "1.5");
  assert(compareArray(sample.subarray(0, -0.6), []), "-0.6");
  assert(compareArray(sample.subarray(0, -1.1), [40n, 41n, 42n]), "-1.1");
  assert(compareArray(sample.subarray(0, -1.5), [40n, 41n, 42n]), "-1.5");

  assert(compareArray(sample.subarray(0, "3"), [40n, 41n, 42n]), "string");
  assert(
    compareArray(
      sample.subarray(0, obj),
      [40n, 41n]
    ),
    "object"
  );
});
