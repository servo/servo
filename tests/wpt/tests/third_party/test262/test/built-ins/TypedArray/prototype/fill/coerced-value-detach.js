// Copyright (C) 2020 Google. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.fill
description: >
  Security Throws a TypeError if value coercion detaches the buffer
info: |
  22.2.3.8 %TypedArray%.prototype.fill (value [ , start [ , end ] ] )

  5. Otherwise, set value to ? ToNumber(value).
  ...
  10. If IsDetachedBuffer(O.[[ViewedArrayBuffer]]) is true, throw a TypeError exception.
 
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA(makeCtorArg(10));

  function detachAndReturnIndex(){
    $DETACHBUFFER(sample.buffer);
    return 0x77;
  }

  assert.throws(TypeError, function() {
    sample.fill({valueOf: detachAndReturnIndex}, 0, 10);
  }, "Detachment when coercing value should throw TypeError");
}, null, null, ["immutable"]);
