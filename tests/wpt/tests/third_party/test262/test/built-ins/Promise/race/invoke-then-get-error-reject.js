// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  Error thrown when accessing the instance's `then` method (rejecting promise)
esid: sec-promise.race
info: |
    11. Let result be PerformPromiseRace(iteratorRecord, C, promiseCapability).
    12. If result is an abrupt completion,
        a. If iteratorRecord.[[done]] is false, let result be
           IteratorClose(iterator, result).
        b. IfAbruptRejectPromise(result, promiseCapability).

    [...]

    25.4.4.3.1 Runtime Semantics: PerformPromiseRace

    1. Repeat
        [...]
        j. Let result be Invoke(nextPromise, "then",
           «promiseCapability.[[Resolve]], promiseCapability.[[Reject]]»).
        k. ReturnIfAbrupt(result).
flags: [async]
---*/

var promise = new Promise(function() {});
var error = new Test262Error();

Object.defineProperty(promise, 'then', {
  get: function() {
    throw error;
  }
});

Promise.race([promise]).then(function() {
  throw new Test262Error('The promise should be rejected');
}, function(reason) {
  assert.sameValue(reason, error);
}).then($DONE, $DONE);
