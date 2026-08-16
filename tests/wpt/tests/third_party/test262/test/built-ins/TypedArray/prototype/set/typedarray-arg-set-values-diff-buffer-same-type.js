// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-typedarray-offset
description: >
  Set values from different instances using the different buffer and same
  constructor. srcBuffer values are cached.
info: |
  22.2.3.23.2 %TypedArray%.prototype.set(typedArray [ , offset ] )

  1. Assert: typedArray has a [[TypedArrayName]] internal slot. If it does not,
  the definition in 22.2.3.23.1 applies.
  ...
  23. If SameValue(srcBuffer, targetBuffer) is true, then
    ...
  24. Else, let srcByteIndex be srcByteOffset.
  ...
  27. If SameValue(srcType, targetType) is true, then,
    a. NOTE: If srcType and targetType are the same, the transfer must be
    performed in a manner that preserves the bit-level encoding of the source
    data.
    b. Repeat, while targetByteIndex < limit
      i. Let value be GetValueFromBuffer(srcBuffer, srcByteIndex, "Uint8").
      ii. Perform SetValueInBuffer(targetBuffer, targetByteIndex, "Uint8",
      value).
  ...
  29. Return undefined.
includes: [testTypedArray.js, compareArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample, result;
  var src = new TA(makeCtorArg([42, 43]));

  sample = new TA(makeCtorArg([1, 2, 3, 4]));
  result = sample.set(src, 1);
  assert(compareArray(sample, [1, 42, 43, 4]), "offset: 1, result: " + sample);
  assert.sameValue(result, undefined, "returns undefined");

  sample = new TA(makeCtorArg([1, 2, 3, 4]));
  result = sample.set(src, 0);
  assert(compareArray(sample, [42, 43, 3, 4]), "offset: 0, result: " + sample);
  assert.sameValue(result, undefined, "returns undefined");

  sample = new TA(makeCtorArg([1, 2, 3, 4]));
  result = sample.set(src, 2);
  assert(compareArray(sample, [1, 2, 42, 43]), "offset: 2, result: " + sample);
  assert.sameValue(result, undefined, "returns undefined");
}, null, null, ["immutable"]);
