// Copyright (C) 2019 Leo Balter, 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  Error thrown when accessing the instance's `then` method (rejecting Promise)
esid: sec-promise.any
info: |
  5. Let result be PerformPromiseAny(iteratorRecord, C, promiseCapability).
  6. If result is an abrupt completion, then
    a. If iteratorRecord.[[Done]] is false, set result to IteratorClose(iteratorRecord, result).
    b. IfAbruptRejectPromise(result, promiseCapability).

  Runtime Semantics: PerformPromiseAny

  r. Perform ? Invoke(nextPromise, "then", « resultCapability.[[Resolve]], rejectElement »).

flags: [async]
features: [Promise.any, arrow-function]
---*/
let error = new Test262Error();
let promise = Promise.resolve();

Object.defineProperty(promise, 'then', {
  get() {
    throw error;
  }
});

Promise.any([promise]).then(() => {
  $DONE('The promise should be rejected, but was resolved');
}, (reason) => {
  assert.sameValue(reason, error);
}).then($DONE, $DONE);
