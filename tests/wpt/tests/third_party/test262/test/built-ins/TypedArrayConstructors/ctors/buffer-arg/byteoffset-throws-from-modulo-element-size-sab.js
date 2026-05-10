// Copyright (C) 2016 the V8 project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-buffer-byteoffset-length
description: >
  Throws a RangeError if ToInteger(byteOffset) modulo elementSize is not 0
info: |
  22.2.4.5 TypedArray ( buffer [ , byteOffset [ , length ] ] )

  This description applies only if the TypedArray function is called with at
  least one argument and the Type of the first argument is Object and that
  object has an [[ArrayBufferData]] internal slot.


  Let elementSize be the Number value of the Element Size value in Table 56 for constructorName.*

  ...
  If offset modulo elementSize ≠ 0, throw a RangeError exception.
  ...

  * Int8Array, Uint8Array, Uint8ClampedArray all have element size 1, so will never fail.

includes: [testTypedArray.js]
features: [SharedArrayBuffer, TypedArray]
---*/

var buffer = new SharedArrayBuffer(8);

testWithTypedArrayConstructors(function(TA) {
  assert.throws(RangeError, function() {
    new TA(buffer, 7);
  });
}, floatArrayConstructors.concat([ Int32Array, Int16Array, Uint32Array, Uint16Array ]));

