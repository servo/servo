// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.slice
description: Infinity values on start and end
includes: [testTypedArray.js, compareArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([40, 41, 42, 43]));

  assert(
    compareArray(sample.slice(-Infinity), [40, 41, 42, 43]),
    "start == -Infinity"
  );
  assert(
    compareArray(sample.slice(Infinity), []),
    "start == Infinity"
  );
  assert(
    compareArray(sample.slice(0, -Infinity), []),
    "end == -Infinity"
  );
  assert(
    compareArray(sample.slice(0, Infinity), [40, 41, 42, 43]),
    "end == Infinity"
  );
});
