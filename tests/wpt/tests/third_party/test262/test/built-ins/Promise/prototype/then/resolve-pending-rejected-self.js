// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Resolving with a reference to the promise itself from a pending promise that is later rejected
es6id: 25.4.5.3
info: |
    [...]
    7. Return PerformPromiseThen(promise, onFulfilled, onRejected,
       resultCapability).

    25.4.5.3.1 PerformPromiseThen
    [...]
    7. If the value of promise's [[PromiseState]] internal slot is "pending",
       [...]
       b. Append rejectReaction as the last element of the List that is the
          value of promise's [[PromiseRejectReactions]] internal slot.
    [...]

    25.4.1.3.2 Promise Resolve Functions
    [...]
    6. If SameValue(resolution, promise) is true, then
       a. Let selfResolutionError be a newly created TypeError object.
       b. Return RejectPromise(promise, selfResolutionError).
flags: [async]
---*/

var reject;
var p1 = new Promise(function(_, _reject) {
  reject = _reject;
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

reject();
