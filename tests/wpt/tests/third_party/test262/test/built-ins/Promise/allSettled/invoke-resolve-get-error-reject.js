// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Error retrieving the constructor's `resolve` method (rejecting promise)
esid: sec-promise.allsettled
info: |
  6. Let result be PerformPromiseAllSettled(iteratorRecord, C, promiseCapability).
  7. If result is an abrupt completion, then
    a. If iteratorRecord.[[Done]] is false, set result to IteratorClose(iteratorRecord, result).
    b. IfAbruptRejectPromise(result, promiseCapability).

  Runtime Semantics: PerformPromiseAllSettled

  6. Repeat
    ...
    i. Let nextPromise be ? Invoke(constructor, "resolve", « nextValue »).
flags: [async]
features: [Promise.allSettled]
---*/

var error = new Test262Error();
Object.defineProperty(Promise, 'resolve', {
  get() {
    throw error;
  }
});

Promise.allSettled([new Promise(function() {})]).then(function() {
  throw new Test262Error('The promise should be rejected');
}, function(reason) {
  assert.sameValue(reason, error);
}).then($DONE, $DONE);
