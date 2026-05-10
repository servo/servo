// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Rejecting through deferred invocation of the provided resolving function,
  captured in a queued job.
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
  7. If promise.[[PromiseState]] is "pending", then
    ...
    b. Append rejectReaction as the last element of the List that is
      promise.[[PromiseRejectReactions]].
  ...
flags: [async]
---*/

var thenable = Promise.resolve();
var returnValue = null;
var reject;
var p = new Promise(function(_, _reject) {
  reject = _reject;
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

returnValue = reject(thenable);

assert.sameValue(returnValue, undefined, '"reject" function return value');
