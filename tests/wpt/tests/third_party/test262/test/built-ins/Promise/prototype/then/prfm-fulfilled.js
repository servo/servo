// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.4.5.3
description: PerformPromiseThen on a fulfilled promise
info: |
    7. Return PerformPromiseThen(promise, onFulfilled, onRejected,
       resultCapability).

    25.4.5.3.1 PerformPromiseThen

    [...]
    8. Else if the value of promise's [[PromiseState]] internal slot is
       "fulfilled",
       a. Let value be the value of promise's [[PromiseResult]] internal slot.
       b. Perform EnqueueJob("PromiseJobs", PromiseReactionJob,
          «fulfillReaction, value»).
    [...]
flags: [async]
---*/

var value = {};
var p = new Promise(function(resolve) {
  resolve(value);
});

p.then(function(x) {
  if (x !== value) {
    $DONE('The `onFulfilled` handler should be invoked with the promise result.');
    return;
  }
  $DONE();
}, function() {
  $DONE('The `onRejected` handler should not be invoked.');
});
