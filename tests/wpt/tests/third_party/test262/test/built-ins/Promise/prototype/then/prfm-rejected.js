// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.4.5.3
description: PerformPromiseThen on a rejected promise
info: |
    7. Return PerformPromiseThen(promise, onFulfilled, onRejected,
       resultCapability).

    25.4.5.3.1 PerformPromiseThen

    [...]
    9. Else if the value of promise's [[PromiseState]] internal slot is
       "rejected",
       a. Let reason be the value of promise's [[PromiseResult]] internal slot.
       b. Perform EnqueueJob("PromiseJobs", PromiseReactionJob,
          «rejectReaction, reason»).
    [...]
flags: [async]
---*/

var value = {};
var p = new Promise(function(_, reject) {
  reject(value);
});

p.then(function() {
  $DONE('The `onFulfilled` handler should not be invoked.');
}, function(x) {
  if (x !== value) {
    $DONE('The `onRejected` handler should be invoked with the promise result.');
    return;
  }
  $DONE();
});
