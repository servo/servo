// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-typedarray-offset
description: >
  Set converted values from different buffer and different type instances
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
    ...
  28. Else,
    a. Repeat, while targetByteIndex < limit
      i. Let value be GetValueFromBuffer(srcBuffer, srcByteIndex, srcType).
      ii. Perform SetValueInBuffer(targetBuffer, targetByteIndex, targetType,
      value).
includes: [byteConversionValues.js, testTypedArray.js]
features: [TypedArray]
---*/

testTypedArrayConversions(byteConversionValues, function(TA, value, expected, initial) {
  if (TA === Float64Array) {
    return;
  }
  var src = new Float64Array([value]);
  var target = new TA([initial]);

  target.set(src);

  assert.sameValue(target[0], expected);
});
