// Copyright (C) 2016 the V8 project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-buffer-byteoffset-length
description: >
  Return new typedArray from undefined offset and length
info: |
  22.2.4.5 TypedArray ( buffer [ , byteOffset [ , length ] ] )

  This description applies only if the TypedArray function is called with at
  least one argument and the Type of the first argument is Object and that
  object has an [[ArrayBufferData]] internal slot.
includes: [testTypedArray.js]
features: [SharedArrayBuffer, TypedArray]
---*/

testWithTypedArrayConstructors(function(TA) {
  var bpe = TA.BYTES_PER_ELEMENT;

  var buffer1 = new SharedArrayBuffer(bpe * 4);
  var ta1 = new TA(buffer1);
  assert.sameValue(ta1.length, 4);
  assert.sameValue(ta1.buffer, buffer1);
  assert.sameValue(ta1.constructor, TA);
  assert.sameValue(Object.getPrototypeOf(ta1), TA.prototype);

  var buffer2 = new SharedArrayBuffer(0);
  var ta2 = new TA(buffer2);
  assert.sameValue(ta2.length, 0);
  assert.sameValue(ta2.buffer, buffer2);
  assert.sameValue(ta2.constructor, TA);
  assert.sameValue(Object.getPrototypeOf(ta2), TA.prototype);
}, null, ["passthrough"]);
