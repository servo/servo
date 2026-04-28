// Copyright (C) 2018 Amal Hussein. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-validatesharedintegertypedarray
description: >
  Throws a TypeError if typedArray arg is not a BigInt64Array
info: |
  Atomics.wait( typedArray, index, value, timeout )

  1.Let buffer be ? ValidateSharedIntegerTypedArray(typedArray, true).
  ...


  ValidateSharedIntegerTypedArray(typedArray [ , waitable ] )

  ...
  5. If waitable is true, then
      a. If typeName is not "BigInt64Array",
      throw a TypeError exception.

features: [Atomics, BigInt, SharedArrayBuffer]
---*/

const i64a = new BigUint64Array(
  new SharedArrayBuffer(BigUint64Array.BYTES_PER_ELEMENT)
);

const poisoned = {
  valueOf: function() {
    throw new Test262Error('should not evaluate this code');
  }
};

assert.throws(TypeError, function() {
  Atomics.wait(i64a, 0, 0n, 0);
});

assert.throws(TypeError, function() {
  Atomics.wait(i64a, poisoned, poisoned, poisoned);
});
