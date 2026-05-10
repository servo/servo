// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-%typedarray%.prototype.buffer
description: >
  Return buffer from [[ViewedArrayBuffer]] internal slot
info: |
  22.2.3.1 get %TypedArray%.prototype.buffer

  ...
  4. Let buffer be the value of O's [[ViewedArrayBuffer]] internal slot.
  5. Return buffer.
includes: [testTypedArray.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA) {
  var buffer = new ArrayBuffer(TA.BYTES_PER_ELEMENT);
  var ta = new TA(buffer);

  assert.sameValue(ta.buffer, buffer);
}, null, ["passthrough"]);
