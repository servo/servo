// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Error thrown when invoking the instance's `then` method (rejecting Promise)
esid: sec-promise.allsettled
info: |
  6. Let result be PerformPromiseAllSettled(iteratorRecord, C, promiseCapability).
  7. If result is an abrupt completion, then
    a. If iteratorRecord.[[Done]] is false, set result to IteratorClose(iteratorRecord, result).
    b. IfAbruptRejectPromise(result, promiseCapability).

  Runtime Semantics: PerformPromiseAllSettled

  z. Perform ? Invoke(nextPromise, "then", « resolveElement, rejectElement »).
flags: [async]
features: [Promise.allSettled]
---*/

var promise = new Promise(function() {});
var error = new Test262Error();

promise.then = function() {
  throw error;
};

Promise.allSettled([promise]).then(function() {
  throw new Test262Error('The promise should be rejected');
}, function(reason) {
  assert.sameValue(reason, error);
}).then($DONE, $DONE);
