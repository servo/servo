// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: A normal completion should trigger promise fulfillment
es6id: 25.4.5.3
info: |
    [...]
    7. Return PerformPromiseThen(promise, onFulfilled, onRejected,
       resultCapability).

    25.4.5.3.1 PerformPromiseThen
    [...]
    8. Else if the value of promise's [[PromiseState]] internal slot is
       "fulfilled",
       a. Let value be the value of promise's [[PromiseResult]] internal slot.
       b. EnqueueJob("PromiseJobs", PromiseReactionJob, «fulfillReaction,
          value»).

    25.4.2.1 PromiseReactionJob
    [...]
    7. If handlerResult is an abrupt completion, then
       a. Let status be Call(promiseCapability.[[Reject]], undefined,
          «handlerResult.[[value]]»).
       b. NextJob Completion(status).
    8. Let status be Call(promiseCapability.[[Resolve]], undefined,
       «handlerResult.[[value]]»).
    9. NextJob Completion(status).
flags: [async]
---*/

var value = {};
var p1 = new Promise(function(resolve) {
  resolve();
});
var p2;

p2 = p1.then(function() {
  return value;
}, function() {});

p2.then(function(x) {
  if (x !== value) {
    $DONE('The `onFulfilled` handler should be invoked with the promise result.');
    return;
  }

  $DONE();
}, function() {
  $DONE('The `onRejected` handler should not be invoked.');
});
