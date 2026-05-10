// Copyright (C) 2018 Amal Hussein. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-atomics.wait
description: >
  A null value for bufferData (detached) throws a TypeError
info: |
  Atomics.wait( typedArray, index, value, timeout )

  Let buffer be ? ValidateIntegerTypedArray(typedArray, true).
  ...

    Let buffer be ? ValidateTypedArray(typedArray).
    ...

      If IsDetachedBuffer(buffer) is true, throw a TypeError exception.
      ...

        If arrayBuffer.[[ArrayBufferData]] is null, return true.

includes: [detachArrayBuffer.js]
features: [ArrayBuffer, Atomics, TypedArray]
---*/

const i32a = new Int32Array(
  new ArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4)
);

const poisoned = {
  valueOf: function() {
    throw new Test262Error('should not evaluate this code');
  }
};

try {
  $DETACHBUFFER(i32a.buffer); // Detaching a non-shared ArrayBuffer sets the [[ArrayBufferData]] value to null
} catch (error) {
  throw new Test262Error(`An unexpected error occurred when detaching ArrayBuffer: ${error.message}`);
}

assert.throws(TypeError, function() {
  Atomics.wait(i32a, poisoned, poisoned, poisoned);
});
