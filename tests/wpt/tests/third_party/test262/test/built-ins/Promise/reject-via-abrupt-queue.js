// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Rejecting through an abrupt completion - captured in a queued job
esid: sec-promise-executor
info: |
  25.4.3.1 Promise ( executor )

  ...
  9. Let completion be Call(executor, undefined, « resolvingFunctions.[[Resolve]],
    resolvingFunctions.[[Reject]] »).
  10. If completion is an abrupt completion, then
    a. Perform ? Call(resolvingFunctions.[[Reject]], undefined, « completion.[[Value]] »).
  11. Return promise.

  25.4.1.3.1 Promise Reject Functions

  ...
  6. Return RejectPromise(promise, reason).

  25.4.5.3.1 PerformPromiseThen ( promise, onFulfilled, onRejected, resultCapability )

  ...
  4. If IsCallable(onRejected) is false, then
    a. Set onRejected to undefined.
  ...
  6. Let rejectReaction be the PromiseReaction { [[Capability]]: resultCapability,
    [[Type]]: "Reject", [[Handler]]: onRejected }.
  ...
  9. Else,
    a. Assert: The value of promise.[[PromiseState]] is "rejected".
    ...
    d. Perform EnqueueJob("PromiseJobs", PromiseReactionJob, « rejectReaction, reason »).
flags: [async]
---*/

var thenable = Promise.resolve();
var p = new Promise(function() {
  throw thenable;
});

p.then(function() {
  $DONE('The promise should not be fulfilled.');
}).then(function() {
  $DONE('The promise should not be fulfilled.');
}, function(x) {
  if (x !== thenable) {
    $DONE('The promise should be rejected with the resolution value.');
    return;
  }
  $DONE();
});
