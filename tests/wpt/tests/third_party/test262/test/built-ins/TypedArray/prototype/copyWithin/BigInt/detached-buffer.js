// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.copywithin
description: Throws a TypeError if this has a detached buffer
info: |
  22.2.3.5 %TypedArray%.prototype.copyWithin (target, start [, end ] )

  This function is not generic. ValidateTypedArray is applied to the this value
  prior to evaluating the algorithm. If its result is an abrupt completion that
  exception is thrown instead of evaluating the algorithm.

  22.2.3.5.1 Runtime Semantics: ValidateTypedArray ( O )

  ...
  5. If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
  ...
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [BigInt, TypedArray]
---*/

var obj = {
  valueOf: function() {
    throw new Test262Error();
  }
};

testWithBigIntTypedArrayConstructors(function(TA) {
  var sample = new TA(1);
  $DETACHBUFFER(sample.buffer);
  assert.throws(TypeError, function() {
    sample.copyWithin(obj, obj);
  });
}, null, ["passthrough"]);
