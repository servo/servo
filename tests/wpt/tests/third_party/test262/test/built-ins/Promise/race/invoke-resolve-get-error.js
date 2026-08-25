// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.race
description: >
  Promise.resolve is retrieved before GetIterator call (abrupt lookup).
info: |
  Promise.race ( iterable )

  [...]
  3. Let promiseResolve be GetPromiseResolve(C).
  4. IfAbruptRejectPromise(promiseResolve, promiseCapability).

  GetPromiseResolve ( promiseConstructor )

  [...]
  2. Let promiseResolve be ? Get(promiseConstructor, "resolve").
flags: [async]
features: [Symbol.iterator]
---*/

const iter = {
  get [Symbol.iterator]() {
    throw new Test262Error('unreachable');
  },
};

const resolveError = { name: 'MyError' };
Object.defineProperty(Promise, 'resolve', {
  get() {
    throw resolveError;
  },
});

Promise.race(iter).then(() => {
  throw new Test262Error('The promise should be rejected, but it was resolved');
}, (reason) => {
  assert.sameValue(reason, resolveError);
}).then($DONE, $DONE);
