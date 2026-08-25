// Copyright (C) 2019 Google. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.copywithin
description: >
  SECURITY: end argument is coerced to an integer values
  causing array detachment, but the value is still defined
  by a prototype
info: |
  22.2.3.5%TypedArray%.prototype.copyWithin ( target, start [ , end ] )
  ...
  8. If end is undefined, let relativeEnd be len; else let relativeEnd be ? ToInteger(end).
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

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var ta;
  var array = [];

  function detachAndReturnIndex(){
      $DETACHBUFFER(ta.buffer);
      Object.setPrototypeOf(ta, array);
      return 101;
  }

  array.length = 10000; // big arrays are more likely to cause a crash if they are accessed after they are freed
  array.fill(7, 0);
  ta = new TA(makeCtorArg(array));
  assert.throws(TypeError, function(){
    ta.copyWithin(0, 100, {valueOf : detachAndReturnIndex});
  }, "should throw TypeError as array is detached");
}, null, null, ["immutable"]);
