// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-buffer-byteoffset-length
description: >
  Return new typedArray from defined length
info: |
  22.2.4.5 TypedArray ( buffer [ , byteOffset [ , length ] ] )

  This description applies only if the TypedArray function is called with at
  least one argument and the Type of the first argument is Object and that
  object has an [[ArrayBufferData]] internal slot.

includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA) {
  var bpe = TA.BYTES_PER_ELEMENT;
  var length = 4;
  var buffer = new ArrayBuffer(bpe * length * 4);

  var ta1 = new TA(buffer, 0, length);
  assert.sameValue(ta1.length, length);
  assert.sameValue(ta1.buffer, buffer);
  assert.sameValue(ta1.constructor, TA);
  assert.sameValue(Object.getPrototypeOf(ta1), TA.prototype);

  var ta2 = new TA(buffer, 0, 0);
  assert.sameValue(ta2.length, 0);
  assert.sameValue(ta2.buffer, buffer);
  assert.sameValue(ta2.constructor, TA);
  assert.sameValue(Object.getPrototypeOf(ta2), TA.prototype);
}, null, ["passthrough"]);
