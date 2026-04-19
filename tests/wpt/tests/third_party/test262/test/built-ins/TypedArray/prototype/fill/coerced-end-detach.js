// Copyright (C) 2020 Google. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.fill
description: >
  Security Throws a TypeError if end coercion detaches the buffer
info: |
  22.2.3.8 %TypedArray%.prototype.fill (value [ , start [ , end ] ] )

  9. If relativeEnd < 0, let final be max((len + relativeEnd), 0); else let final be min(relativeEnd, len).
  10. If IsDetachedBuffer(O.[[ViewedArrayBuffer]]) is true, throw a TypeError exception.
 
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA) {
  var sample = new TA(10);

  function detachAndReturnIndex(){
    $DETACHBUFFER(sample.buffer);
    return 10;
  }

  assert.throws(TypeError, function() {
    sample.fill(0x77, 0, {valueOf: detachAndReturnIndex});
  }, "Detachment when coercing end should throw TypeError");
}, null, ["passthrough"]);
