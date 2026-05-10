// Copyright (C) 2018 Amal Hussein. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.wait
description: >
  False timeout arg should result in an +0 timeout
info: |
  Atomics.wait( typedArray, index, value, timeout )

  4. Let q be ? ToNumber(timeout).

    Boolean -> If argument is true, return 1. If argument is false, return +0.

features: [Atomics, BigInt, SharedArrayBuffer, Symbol, Symbol.toPrimitive, TypedArray]
flags: [CanBlockIsTrue]
---*/

const i64a = new BigInt64Array(
  new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 4)
);

const valueOf = {
  valueOf: function() {
    return false;
  }
};

const toPrimitive = {
  [Symbol.toPrimitive]: function() {
    return false;
  }
};

assert.sameValue(
  Atomics.wait(i64a, 0, 0n, false),
  "timed-out",
  'Atomics.wait(i64a, 0, 0n, false) returns "timed-out"'
);
assert.sameValue(
  Atomics.wait(i64a, 0, 0n, valueOf),
  "timed-out",
  'Atomics.wait(i64a, 0, 0n, valueOf) returns "timed-out"'
);
assert.sameValue(
  Atomics.wait(i64a, 0, 0n, toPrimitive),
  "timed-out",
  'Atomics.wait(i64a, 0, 0n, toPrimitive) returns "timed-out"'
);
