// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-%typedarray%.prototype.byteoffset
description: Returns 0 if the instance has a detached buffer
info: |
  22.2.3.3 get %TypedArray%.prototype.byteOffset

  ...
  4. Let buffer be the value of O's [[ViewedArrayBuffer]] internal slot.
  5. If IsDetachedBuffer(buffer) is true, return 0.
  ...
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA) {
  var buffer = new ArrayBuffer(128);
  var sample = new TA(buffer, 8, 1);
  $DETACHBUFFER(sample.buffer);
  assert.sameValue(sample.byteOffset, 0);
}, null, ["passthrough"]);
