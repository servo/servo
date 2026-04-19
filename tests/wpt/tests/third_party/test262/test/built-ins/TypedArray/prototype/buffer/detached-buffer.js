// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-%typedarray%.prototype.buffer
description: The getter method does not throw with a detached buffer
info: |
  22.2.3.1 get %TypedArray%.prototype.buffer

  ...
  4. Let buffer be the value of O's [[ViewedArrayBuffer]] internal slot.
  5. Return buffer.
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA) {
  var buffer = new ArrayBuffer(8);
  var sample = new TA(buffer, 0, 1);
  $DETACHBUFFER(sample.buffer);
  assert.sameValue(sample.buffer, buffer);
}, null, ["passthrough"]);
