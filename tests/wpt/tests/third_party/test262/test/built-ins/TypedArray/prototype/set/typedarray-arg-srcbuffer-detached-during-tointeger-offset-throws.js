// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.set-typedarray-offset
description: >
  Throws a TypeError if srcBuffer is detached on ToInteger(offset)
info: |
  22.2.3.23.2 %TypedArray%.prototype.set(typedArray [ , offset ] )

  1. Assert: typedArray has a [[TypedArrayName]] internal slot. If it does not,
  the definition in 22.2.3.23.1 applies.
  ...
  6. Let targetOffset be ? ToInteger(offset).
  ...
  11. Let srcBuffer be the value of typedArray's [[ViewedArrayBuffer]] internal
  slot.
  12. If IsDetachedBuffer(srcBuffer) is true, throw a TypeError exception.
  ...
includes: [testTypedArray.js, detachArrayBuffer.js]
features: [TypedArray]
---*/

testWithTypedArrayConstructors(function(TA, makeCtorArg) {
  var sample = new TA();
  var target = new TA(makeCtorArg(0));
  var calledOffset = 0;
  var obj = {
    valueOf: function() {
      $DETACHBUFFER(target.buffer);
      calledOffset += 1;
    }
  };

  assert.throws(TypeError, function() {
    sample.set(target, obj);
  });

  assert.sameValue(calledOffset, 1);
}, null, null, ["immutable"]);
