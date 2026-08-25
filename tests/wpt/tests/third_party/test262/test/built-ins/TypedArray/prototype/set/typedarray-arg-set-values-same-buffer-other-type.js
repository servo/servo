// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-typedarray-offset
description: >
  Set values from different instances using the same buffer and different
  constructor.
info: |
  22.2.3.23.2 %TypedArray%.prototype.set(typedArray [ , offset ] )

  1. Assert: typedArray has a [[TypedArrayName]] internal slot. If it does not,
  the definition in 22.2.3.23.1 applies.
  ...
  23. If SameValue(srcBuffer, targetBuffer) is true, then
    a. Let srcBuffer be ? CloneArrayBuffer(srcBuffer, srcByteOffset, srcLength,
    %ArrayBuffer%).
    b. NOTE: %ArrayBuffer% is used to clone srcBuffer because is it known to not
    have any observable side-effects.
    ...
  ...
  27. If SameValue(srcType, targetType) is true, then,
    ...
  28. Else,
    a. Repeat, while targetByteIndex < limit
      i. Let value be GetValueFromBuffer(srcBuffer, srcByteIndex, srcType).
      ii. Perform SetValueInBuffer(targetBuffer, targetByteIndex, targetType,
      value).
  ...
  29. Return undefined.
includes: [testTypedArray.js, compareArray.js]
features: [TypedArray]
---*/

var expected = {
  Float64Array: [1.0000002464512363, 42, 1.875, 4, 5, 6, 7, 8],
  Float32Array: [0, 42, 512.0001220703125, 4, 5, 6, 7, 8],
  Float16Array: [0, 42, 513, 4, 5, 6, 7, 8],
  Int32Array: [1109917696, 42, 0, 4, 5, 6, 7, 8],
  Int16Array: [0, 42, 0, 4, 5, 6, 7, 8],
  Int8Array: [0, 42, 0, 66, 5, 6, 7, 8],
  Uint32Array: [1109917696, 42, 0, 4, 5, 6, 7, 8],
  Uint16Array: [0, 42, 0, 4, 5, 6, 7, 8],
  Uint8Array: [0, 42, 0, 66, 5, 6, 7, 8],
  Uint8ClampedArray: [0, 42, 0, 66, 5, 6, 7, 8]
};

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var other = TA === Float32Array ? Float64Array : Float32Array;

  var sample = new TA(makeCtorArg([1, 2, 3, 4, 5, 6, 7, 8]));
  var src = new other(sample.buffer, 0, 2);

  // Reflect changes on sample object
  src[0] = 42;

  var result = sample.set(src, 1);

  assert(compareArray(sample, expected[TA.name]), sample);
  assert.sameValue(result, undefined, "returns undefined");
}, null, null, ["immutable"]);
