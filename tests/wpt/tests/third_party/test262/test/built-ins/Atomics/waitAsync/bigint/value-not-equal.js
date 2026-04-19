// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-atomics.waitasync
description: >
  Returns "not-equal" when value arg does not match an index in the typedArray
info: |
  Atomics.waitAsync( typedArray, index, value, timeout )

  1. Return DoWait(async, typedArray, index, value, timeout).

  DoWait ( mode, typedArray, index, value, timeout )

  16. Let w be ! AtomicLoad(typedArray, i).
  17. If v is not equal to w, then
    a. Perform LeaveCriticalSection(WL).
    b. If mode is sync, then
      i. Return the String "not-equal".
    c. Perform ! Call(capability.[[Resolve]], undefined, « "not-equal" »).
    d. Return promiseCapability.[[Promise]].

flags: [async]
features: [Atomics.waitAsync, SharedArrayBuffer, TypedArray, BigInt, computed-property-names, Symbol, Symbol.toPrimitive, Atomics, arrow-function]
---*/
assert.sameValue(typeof Atomics.waitAsync, 'function', 'The value of `typeof Atomics.waitAsync` is "function"');
const i64a = new BigInt64Array(new SharedArrayBuffer(BigInt64Array.BYTES_PER_ELEMENT * 4));

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

Promise.all([Atomics.store(i64a, 0, 42n), Atomics.waitAsync(i64a, 0, 0n).value]).then(outcomes => {
  assert.sameValue(outcomes[0], 42n, 'The value of outcomes[0] is 42n');
  assert.sameValue(outcomes[1], 'not-equal', 'The value of outcomes[1] is "not-equal"');
}).then($DONE, $DONE);
