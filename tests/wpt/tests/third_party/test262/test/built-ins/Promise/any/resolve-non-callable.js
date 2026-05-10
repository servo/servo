// Copyright (C) 2020 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.any
description: >
  Promise.resolve is retrieved before GetIterator call (non-callable).
info: |
  Promise.any ( iterable )

  [...]
  3. Let promiseResolve be GetPromiseResolve(C).
  4. IfAbruptRejectPromise(promiseResolve, promiseCapability).

  GetPromiseResolve ( promiseConstructor )

  [...]
  2. Let promiseResolve be ? Get(promiseConstructor, "resolve").
  3. If IsCallable(promiseResolve) is false, throw a TypeError exception.
flags: [async]
features: [Promise.any, Symbol.iterator]
---*/

const iter = { 
  get [Symbol.iterator]() {
    throw new Test262Error("unreachable");
  },
};

Promise.resolve = "certainly not callable";

Promise.any(iter).then(() => {
  throw new Test262Error("The promise should be rejected, but it was resolved");
}, (reason) => {
  assert(reason instanceof TypeError);
}).then($DONE, $DONE);
