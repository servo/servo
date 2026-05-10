// Copyright (C) 2020 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.notify
description: >
  Atomics.notify throws on non-shared integer TypedArrays
features: [ArrayBuffer, Atomics, BigInt, TypedArray]
---*/

const nonsab = new ArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 8);

const poisoned = {
  valueOf() {
    throw new Test262Error('should not evaluate this code');
  }
};

assert.throws(TypeError, function() {
  Atomics.notify(new BigUint64Array(nonsab), poisoned, poisoned);
});
