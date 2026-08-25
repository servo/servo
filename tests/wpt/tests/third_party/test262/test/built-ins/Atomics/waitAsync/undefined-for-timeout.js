// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.waitasync
description: >
  Undefined timeout arg should result in an infinite timeout
info: |
  Atomics.waitAsync( typedArray, index, value, timeout )

  1. Return DoWait(async, typedArray, index, value, timeout).

  DoWait ( mode, typedArray, index, value, timeout )

  6. Let q be ? ToNumber(timeout).
    ...
    Undefined    Return NaN.

  5.If q is NaN, let t be +∞, else let t be max(q, 0)

flags: [async]
features: [Atomics.waitAsync, SharedArrayBuffer, TypedArray, Atomics, computed-property-names, Symbol, Symbol.toPrimitive, arrow-function]
---*/
assert.sameValue(typeof Atomics.waitAsync, 'function', 'The value of `typeof Atomics.waitAsync` is "function"');
const i32a = new Int32Array(
  new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 4)
);

const valueOf = {
  valueOf() {
    return undefined;
  }
};

const toPrimitive = {
  [Symbol.toPrimitive]() {
    return undefined;
  }
};

Promise.all([
    Atomics.waitAsync(i32a, 0, 0).value,
    Atomics.waitAsync(i32a, 0, 0, undefined).value,
    Atomics.waitAsync(i32a, 0, 0, valueOf).value,
    Atomics.waitAsync(i32a, 0, 0, toPrimitive).value
  ]).then(outcomes => {
    assert.sameValue(outcomes[0], 'ok', 'The value of outcomes[0] is "ok"');
    assert.sameValue(outcomes[1], 'ok', 'The value of outcomes[1] is "ok"');
    assert.sameValue(outcomes[2], 'ok', 'The value of outcomes[2] is "ok"');
    assert.sameValue(outcomes[3], 'ok', 'The value of outcomes[3] is "ok"');
  }).then($DONE, $DONE);

Atomics.notify(i32a, 0);
