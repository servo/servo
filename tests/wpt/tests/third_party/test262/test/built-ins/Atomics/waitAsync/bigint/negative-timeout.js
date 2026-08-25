// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-atomics.waitasync
description: >
  Test that Atomics.waitAsync times out with a negative timeout
info: |
  Atomics.waitAsync( typedArray, index, value, timeout )

  1. Return DoWait(async, typedArray, index, value, timeout).

  DoWait ( mode, typedArray, index, value, timeout )

  6. Let q be ? ToNumber(timeout).

flags: [async]
features: [Atomics.waitAsync, SharedArrayBuffer, TypedArray, Atomics, BigInt, destructuring-binding, arrow-function]
---*/
assert.sameValue(typeof Atomics.waitAsync, 'function', 'The value of `typeof Atomics.waitAsync` is "function"');
const i64a = new BigInt64Array(new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 4));

Promise.all([Atomics.waitAsync(i64a, 0, 0n, -1).value]).then(([outcome]) => {
  assert.sameValue(outcome, 'timed-out', 'The value of `outcome` is "timed-out"');
}).then($DONE, $DONE);
