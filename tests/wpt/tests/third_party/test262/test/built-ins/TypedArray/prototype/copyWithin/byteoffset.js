// Copyright (C) 2021 Chengzhong Wu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.copywithin
description: >
  copyWithin should respect typedarray's byteOffset
info: |
  22.2.3.5%TypedArray%.prototype.copyWithin ( target, start [ , end ] )
  ...
  17. If count > 0, then
    e. Let elementSize be the Element Size value specified in Table 72 for typedArrayName.
    f. Let byteOffset be O.[[ByteOffset]].
    g. Let toByteIndex be to × elementSize + byteOffset.
    h. Let fromByteIndex be from × elementSize + byteOffset.
  ...
includes: [testTypedArray.js, compareArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var ta = new TA(makeCtorArg([0, 1, 2, 3]));
  assert.compareArray(
    new TA(ta.buffer, TA.BYTES_PER_ELEMENT).copyWithin(2, 0),
    [1, 2, 1],
    'copyWithin should respect typedarray\'s byteOffset'
  );

  assert.compareArray(
    ta,
    [0, 1, 2, 1],
    'underlying arraybuffer should have been updated'
  );
});
