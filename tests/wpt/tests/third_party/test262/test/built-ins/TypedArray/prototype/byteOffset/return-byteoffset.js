// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-%typedarray%.prototype.byteoffset
description: >
  Return value from [[ByteOffset]] internal slot
info: |
  22.2.3.3 get %TypedArray%.prototype.byteOffset

  ...
  6. Let offset be the value of O's [[ByteOffset]] internal slot.
  7. Return size.
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var ta1 = new TA();
  assert.sameValue(ta1.byteOffset, 0, "Regular typedArray");

  var offset = 4 * TA.BYTES_PER_ELEMENT;

  var buffer1 = makeCtorArg(8);
  var ta2 = new TA(buffer1, offset);
  assert.sameValue(ta2.byteOffset, offset, "TA(buffer, offset)");

  var buffer2 = makeCtorArg(8);
  var sample = new TA(buffer2, offset);
  var ta3 = new TA(sample);
  assert.sameValue(ta3.byteOffset, 0, "TA(typedArray)");
}, null, ["arraybuffer"]);
