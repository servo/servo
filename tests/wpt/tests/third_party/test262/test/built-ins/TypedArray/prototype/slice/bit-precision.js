// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.slice
description: Preservation of bit-level encoding
info: |
  [...]
  15. Else if count > 0, then
      [...]
      e. NOTE: If srcType and targetType are the same, the transfer must be
         performed in a manner that preserves the bit-level encoding of the
         source data.
      f. Let srcByteOffet be the value of O's [[ByteOffset]] internal slot.
      g. Let targetByteIndex be A's [[ByteOffset]] internal slot.
      h. Let srcByteIndex be (k × elementSize) + srcByteOffet.
      i. Let limit be targetByteIndex + count × elementSize.
      j. Repeat, while targetByteIndex < limit
         i. Let value be GetValueFromBuffer(srcBuffer, srcByteIndex, "Uint8").
         ii. Perform SetValueInBuffer(targetBuffer, targetByteIndex, "Uint8",
             value).
         iii. Increase srcByteIndex by 1.
         iv. Increase targetByteIndex by 1.
includes: [nans.js, compareArray.js, testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function body(FloatArray) {
  var subject = new FloatArray(NaNs);
  var sliced, subjectBytes, slicedBytes;

  sliced = subject.slice();

  subjectBytes = new Uint8Array(subject.buffer);
  slicedBytes = new Uint8Array(sliced.buffer);

  assert(compareArray(subjectBytes, slicedBytes));
}, floatArrayConstructors);
