// Copyright (C) 2018 Amal Hussein. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-atomics.wait
description: >
  A null value for bufferData throws a TypeError
info: |
  Atomics.wait( typedArray, index, value, timeout )

  1.Let buffer be ? ValidateSharedIntegerTypedArray(typedArray, true).
  ...

  ValidateSharedIntegerTypedArray(typedArray [ , onlyInt32 ] )

  ...
  9.If IsSharedArrayBuffer(buffer) is false, throw a TypeError exception.


  IsSharedArrayBuffer( obj )

  ...
  3.If bufferData is null, return false.

includes: [detachArrayBuffer.js]
features: [ArrayBuffer, Atomics, BigInt, TypedArray]
---*/

const i64a = new BigInt64Array(
  new ArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 4)
);
const poisoned = {
  valueOf: function() {
    throw new Test262Error('should not evaluate this code');
  }
};

try {
  $DETACHBUFFER(i64a.buffer); // Detaching a non-shared ArrayBuffer sets the [[ArrayBufferData]] value to null
} catch (error) {
  throw new Test262Error(`An unexpected error occurred when detaching ArrayBuffer: ${error.message}`);
}

assert.throws(TypeError, function() {
  Atomics.wait(i64a, poisoned, poisoned, poisoned);
});
