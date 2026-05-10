// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Resolving with a reference to the promise itself from a rejected promise
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
    6. If SameValue(resolution, promise) is true, then
       a. Let selfResolutionError be a newly created TypeError object.
       b. Return RejectPromise(promise, selfResolutionError).
flags: [async]
---*/

var p1 = new Promise(function(_, reject) {
  reject();
});
var p2;

p2 = p1.then(function() {}, function() {
  return p2;
});

p2.then(function() {
  $DONE('The promise should not be fulfilled.');
}, function(reason) {
  if (!reason) {
    $DONE('The promise should be rejected with a value.');
    return;
  }

  if (reason.constructor !== TypeError) {
    $DONE('The promise should be rejected with a TypeError instance.');
    return;
  }

  $DONE();
});
