// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Rejecting from a pending promise that is later fulfilled
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

    25.4.1.3.1 Promise Reject Functions
    [...]
    6. Return RejectPromise(promise, reason).
flags: [async]
---*/

var resolve;
var thenable = new Promise(function(_resolve) {
  resolve = _resolve;
});
var p1 = new Promise(function(resolve) {
  resolve();
});
var p2;

p2 = p1.then(function() {
  throw thenable;
});

p2.then(function() {
  $DONE('The promise should not be fulfilled.');
}, function(x) {
  if (x !== thenable) {
    $DONE('The promise should be rejected with the resolution value of the provided promise.');
    return;
  }

  $DONE();
});

resolve();
