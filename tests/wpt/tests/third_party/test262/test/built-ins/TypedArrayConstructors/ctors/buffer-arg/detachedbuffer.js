// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-typedarray-buffer-byteoffset-length
description: If TypedArray() is passed a detached buffer, throw
info: |
  22.2.4.5 TypedArray ( buffer [ , byteOffset [ , length ] ] )

  ...
  9. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
  ...
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA) {
  var offset = TA.BYTES_PER_ELEMENT;
  var buffer = new ArrayBuffer(3 * offset);
  $DETACHBUFFER(buffer);
  assert.throws(TypeError, () => new TA(buffer));
}, null, ["passthrough"]);
