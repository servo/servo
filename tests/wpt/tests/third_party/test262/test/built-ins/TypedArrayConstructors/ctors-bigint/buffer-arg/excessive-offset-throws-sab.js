// Copyright (C) 2016 the V8 project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-buffer-byteoffset-length
description: >
  Throws a RangeError if bufferByteLength - ToInteger(byteOffset) < 0
info: |
  22.2.4.5 TypedArray ( buffer [ , byteOffset [ , length ] ] )

  This description applies only if the TypedArray function is called with at
  least one argument and the Type of the first argument is Object and that
  object has an [[ArrayBufferData]] internal slot.

  ...
  13. If length is undefined, then
    a. If bufferByteLength modulo elementSize ≠ 0, throw a RangeError exception.
    b. Let newByteLength be bufferByteLength - offset.
    c. If newByteLength < 0, throw a RangeError exception.
  ...
includes: [testTypedArray.js]
features: [BigInt, SharedArrayBuffer, TypedArray]
---*/

testWithBigIntTypedArrayConstructors(function(TA) {
  var bpe = TA.BYTES_PER_ELEMENT;
  var buffer = new SharedArrayBuffer(bpe);

  assert.throws(RangeError, function() {
    new TA(buffer, bpe * 2);
  });

  assert.throws(RangeError, function() {
    new TA(buffer, bpe * 2, undefined);
  });
}, null, ["passthrough"]);
