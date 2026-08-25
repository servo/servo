// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-%typedarray%.prototype.bytelength
description: >
  Return value from [[ByteLength]] internal slot
info: |
  22.2.3.2 get %TypedArray%.prototype.byteLength

  ...
  6. Let size be the value of O's [[ByteLength]] internal slot.
  7. Return size.
includes: [testTypedArray.js]
features: [BigInt, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA, makeCtorArg) {
  var bytesPerElement = TA.BYTES_PER_ELEMENT;
  var ta1 = new TA();
  assert.sameValue(ta1.byteLength, 0);

  var ta2 = new TA(makeCtorArg(42));
  assert.sameValue(ta2.byteLength, 42 * bytesPerElement);
});
