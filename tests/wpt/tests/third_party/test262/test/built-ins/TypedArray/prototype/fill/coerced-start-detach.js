// Copyright (C) 2020 Google. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.fill
description: >
  Security Throws a TypeError if start coercion detaches the buffer
info: |
  22.2.3.8 %TypedArray%.prototype.fill (value [ , start [ , end ] ] )

  6. Let relativeStart be ? ToInteger(start).
  ...
  10. If IsDetachedBuffer(O.[[ViewedArrayBuffer]]) is true, throw a TypeError exception.
 
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(10));

  function detachAndReturnIndex(){
    $DETACHBUFFER(sample.buffer);
    return 0;
  }

  assert.throws(TypeError, function() {
    sample.fill(0x77, {valueOf: detachAndReturnIndex}, 10);
  }, "Detachment when coercing start should throw TypeError");
}, null, null, ["immutable"]);
