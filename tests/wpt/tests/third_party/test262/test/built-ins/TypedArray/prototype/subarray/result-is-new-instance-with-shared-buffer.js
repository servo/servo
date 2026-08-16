// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.subarray
description: Returns a new instance sharing the same buffer
info: |
  22.2.3.27 %TypedArray%.prototype.subarray( begin , end )

  ...
  17. Return ? TypedArraySpeciesCreate(O, argumentsList).
includes: [testTypedArray.js, compareArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg([40, 41, 42, 43]));
  var buffer = sample.buffer;
  var result = sample.subarray(1);

  assert.notSameValue(result, sample, "returns a new instance");
  assert.sameValue(result.buffer, sample.buffer, "shared buffer");
  assert.sameValue(sample.buffer, buffer, "original buffer is preserved");

  sample[1] = 100;
  assert(
    compareArray(result, [100, 42, 43]),
    "changes on the original sample values affect the new instance"
  );

  result[1] = 111;
  assert(
    compareArray(sample, [40, 100, 111, 43]),
    "changes on the new instance values affect the original sample"
  );
}, null, null, ["immutable"]);
