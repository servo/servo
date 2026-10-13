// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-%typedarray%.prototype.bytelength
description: Returns 0 if the instance has a detached buffer
info: |
  22.2.3.2 get %TypedArray%.prototype.byteLength

  ...
  4. Let buffer be the value of O's [[ViewedArrayBuffer]] internal slot.
  5. If IsDetachedBuffer(buffer) is true, return 0.
  ...
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(1));
  $DETACHBUFFER(sample.buffer);
  assert.sameValue(sample.byteLength, 0);
}, null, null, ["immutable"]);
