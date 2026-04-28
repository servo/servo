// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Error retrieving the constructor's `resolve` method (rejecting promise)
esid: sec-performpromiseall
info: |
    11. Let result be PerformPromiseAll(iteratorRecord, C, promiseCapability).
    12. If result is an abrupt completion,
        a. If iteratorRecord.[[done]] is false, let result be
           IteratorClose(iterator, result).
        b. IfAbruptRejectPromise(result, promiseCapability).

    [...]

    Runtime Semantics: PerformPromiseAll

    ...
    1. Let promiseResolve be ? Get(constructor, `"resolve"`).
    ...
    1. Repeat,
      1. Let next be IteratorStep(iteratorRecord).
      ...
      1. Let nextPromise be ? Call(promiseResolve, constructor, < nextValue >).
flags: [async]
---*/

var error = new Test262Error();
Object.defineProperty(Promise, 'resolve', {
  get: function() {
    throw error;
  }
});

Promise.all([new Promise(function() {})]).then(function() {
  throw new Test262Error('The promise should be rejected');
}, function(reason) {
  assert.sameValue(reason, error);
}).then($DONE, $DONE);
