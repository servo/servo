// Copyright (C) 2016 the V8 project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-buffer-byteoffset-length
description: >
  Return new typedArray from defined length and offset
info: |
  22.2.4.5 TypedArray ( buffer [ , byteOffset [ , length ] ] )

  This description applies only if the TypedArray function is called with at
  least one argument and the Type of the first argument is Object and that
  object has an [[ArrayBufferData]] internal slot.
includes: [testTypedArray.js]
features: [BigInt, SharedArrayBuffer, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA) {
  var offset = TA.BYTES_PER_ELEMENT;
  var buffer = new SharedArrayBuffer(3 * offset);

  var ta1 = new TA(buffer, offset, 2);
  assert.sameValue(ta1.length, 2, "ta1.length");
  assert.sameValue(ta1.buffer, buffer, "ta1.buffer");
  assert.sameValue(ta1.constructor, TA);
  assert.sameValue(Object.getPrototypeOf(ta1), TA.prototype);

  var ta2 = new TA(buffer, offset, 0);
  assert.sameValue(ta2.length, 0, "ta2.length");
  assert.sameValue(ta2.buffer, buffer, "ta2.buffer");
  assert.sameValue(ta2.constructor, TA);
  assert.sameValue(Object.getPrototypeOf(ta2), TA.prototype);
}, null, ["passthrough"]);
