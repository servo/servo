// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Error retrieving the constructor's `resolve` method (promise rejection)
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
        h. Let nextPromise be Invoke(C, "resolve", «nextValue»).
        i. ReturnIfAbrupt(nextPromise).
flags: [async]
---*/

var error = new Test262Error();
Object.defineProperty(Promise, 'resolve', {
  get: function() {
    throw error;
  }
});

Promise.race([new Promise(function() {})]).then(function() {
  throw new Test262Error('The promise should be rejected');
}, function(reason) {
  assert.sameValue(reason, error);
}).then($DONE, $DONE);
