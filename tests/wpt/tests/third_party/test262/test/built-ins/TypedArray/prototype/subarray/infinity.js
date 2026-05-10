// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.subarray
description: Infinity values on begin and end
info: |
  22.2.3.27 %TypedArray%.prototype.subarray( begin , end )
includes: [testTypedArray.js, compareArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([40, 41, 42, 43]));

  assert(
    compareArray(sample.subarray(-Infinity), [40, 41, 42, 43]),
    "begin == -Infinity"
  );
  assert(
    compareArray(sample.subarray(Infinity), []),
    "being == Infinity"
  );
  assert(
    compareArray(sample.subarray(0, -Infinity), []),
    "end == -Infinity"
  );
  assert(
    compareArray(sample.subarray(0, Infinity), [40, 41, 42, 43]),
    "end == Infinity"
  );
});
