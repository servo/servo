// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-buffer-byteoffset-length
description: >
  Throws a RangeError if bufferByteLength modulo elementSize ≠ 0
info: |
  22.2.4.5 TypedArray ( buffer [ , byteOffset [ , length ] ] )

  This description applies only if the TypedArray function is called with at
  least one argument and the Type of the first argument is Object and that
  object has an [[ArrayBufferData]] internal slot.

  ...
  13. If length is undefined, then
    a. If bufferByteLength modulo elementSize ≠ 0, throw a RangeError exception.
  ...
includes: [testTypedArray.js]
features: [TypedArray]
---*/

var buffer = new ArrayBuffer(1);

testWithTypedArrayConstructors(function(TA) {
  assert.throws(RangeError, function() {
    new TA(buffer);
  });

  assert.throws(RangeError, function() {
    new TA(buffer, 0, undefined);
  });
}, floatArrayConstructors.concat([ Int32Array, Int16Array, Uint32Array, Uint16Array ]));
