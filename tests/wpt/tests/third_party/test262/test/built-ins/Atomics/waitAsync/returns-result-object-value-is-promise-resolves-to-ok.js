// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-atomics.waitasync
description: >
  Atomics.waitAsync returns a result object containing a property named "value" whose value is a promise that resolves to "ok" and async is true.
info: |
  Atomics.waitAsync( typedArray, index, value, timeout )

  1. Return DoWait(async, typedArray, index, value, timeout).

  DoWait ( mode, typedArray, index, value, timeout )

  ...
  13. Let promiseCapability be undefined.
  14. If mode is async, then
    a. Set promiseCapability to ! NewPromiseCapability(%Promise%).

  ...
  Perform ! CreateDataPropertyOrThrow(_resultObject_, *"async"*, *true*).
  Perform ! CreateDataPropertyOrThrow(_resultObject_, *"value"*, _promiseCapability_.[[Promise]]).
  Return _resultObject_.

flags: [async]
features: [Atomics.waitAsync, TypedArray, SharedArrayBuffer, destructuring-binding, Atomics, arrow-function]
---*/
assert.sameValue(typeof Atomics.waitAsync, 'function', 'The value of `typeof Atomics.waitAsync` is "function"');

const i32a = new Int32Array(
  new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * 8)
);

let {async, value} = Atomics.waitAsync(i32a, 0, 0, 1000);
assert.sameValue(async, true, 'The value of `async` is true');
assert(value instanceof Promise, 'The result of evaluating `(value instanceof Promise)` is true');
assert.sameValue(
  Object.getPrototypeOf(value),
  Promise.prototype,
  'Object.getPrototypeOf(value) must return the value of Promise.prototype'
);

value.then(outcome => {
  assert.sameValue(outcome, "ok", 'The value of `outcome` is "ok"');
}).then(() => $DONE(), $DONE);

Atomics.add(i32a, 0, 1);
Atomics.notify(i32a, 0, 1);



