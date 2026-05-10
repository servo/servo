// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Resolving with a resolved Promise instance whose `then` method has been overridden from a pending promise that is later fulfilled
es6id: 25.4.5.3
info: |
    [...]
    7. Return PerformPromiseThen(promise, onFulfilled, onRejected,
       resultCapability).

    25.4.5.3.1 PerformPromiseThen
    [...]
    7. If the value of promise's [[PromiseState]] internal slot is "pending",
       a. Append fulfillReaction as the last element of the List that is the
          value of promise's [[PromiseFulfillReactions]] internal slot.
       [...]

    25.4.1.3.2 Promise Resolve Functions
    [...]
    8. Let then be Get(resolution, "then").
    9. If then is an abrupt completion, then
       [...]
    10. Let thenAction be then.[[value]].
    11. If IsCallable(thenAction) is false, then
        [...]
    12. Perform EnqueueJob ("PromiseJobs", PromiseResolveThenableJob,
        «promise, resolution, thenAction»)
flags: [async]
---*/

var value = {};
var resolve;
var thenable = new Promise(function(resolve) {
  resolve();
});
var p1 = new Promise(function(_resolve) {
  resolve = _resolve;
});
var p2;

thenable.then = function(resolve) {
  resolve(value);
};

p2 = p1.then(function() {
  return thenable;
});

p2.then(function(x) {
  if (x !== value) {
    $DONE('The promise should be fulfilled with the resolution value of the provided promise.');
    return;
  }

  $DONE();
}, function() {
  $DONE('The promise should not be rejected.');
});

resolve();
