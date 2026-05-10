// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Resolving with a non-thenable object value from a rejected promise
es6id: 25.4.5.3
info: |
    [...]
    7. Return PerformPromiseThen(promise, onFulfilled, onRejected,
       resultCapability).

    25.4.5.3.1 PerformPromiseThen
    [...]
    9. Else if the value of promise's [[PromiseState]] internal slot is
       "rejected",
       a. Let reason be the value of promise's [[PromiseResult]] internal slot.
       b. Perform EnqueueJob("PromiseJobs", PromiseReactionJob,
          «rejectReaction, reason»).

    25.4.2.1 PromiseReactionJob
    [...]
    8. Let status be Call(promiseCapability.[[Resolve]], undefined,
       «handlerResult.[[value]]»).
    [...]

    25.4.1.3.2 Promise Resolve Functions
    [...]
    8. Let then be Get(resolution, "then").
    9. If then is an abrupt completion, then
       [...]
    10. Let thenAction be then.[[value]].
    11. If IsCallable(thenAction) is false, then
        a. Return FulfillPromise(promise, resolution).
flags: [async]
---*/

var nonThenable = {
  then: null
};
var p1 = new Promise(function(_, reject) {
  reject();
});
var p2;

p2 = p1.then(function() {}, function() {
  return nonThenable;
});

p2.then(function(value) {
  if (value !== nonThenable) {
    $DONE('The promise should be fulfilled with the provided value.');
    return;
  }

  $DONE();
}, function() {
  $DONE('The promise should not be rejected.');
});
