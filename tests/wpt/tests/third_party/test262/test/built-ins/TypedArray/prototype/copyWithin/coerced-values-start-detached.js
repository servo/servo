// Copyright (C) 2019 Google. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.copywithin
description: >
  SECURITY: start argument is coerced to an integer value, which detached
  the array
info: |
  22.2.3.5 %TypedArray%.prototype.copyWithin ( target, start [ , end ] )

  ...
  6. Let relativeStart be ? ToInteger(start).
  ...
  10. Let count be min(final - from, len - to).
  11. If count > 0, then
    a. NOTE: The copying must be performed in a manner that preserves the bit-level encoding of the source data.
    b. Let buffer be O.[[ViewedArrayBuffer]].
    c. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
  ...
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA) {
  var ta;
  function detachAndReturnIndex(){
      $DETACHBUFFER(ta.buffer);
      return 100;
  }

  var array = [];
  array.length = 10000; // big arrays are more likely to cause a crash if they are accessed after they are freed
  array.fill(7, 0);
  ta = new TA(array);
  assert.throws(TypeError, function(){
    ta.copyWithin(0, {valueOf : detachAndReturnIndex}, 1000);
  }, "should throw TypeError as array is detached");
}, null, ["passthrough"]);
