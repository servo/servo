// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.4.5.3
description: PerformPromiseThen on a pending promise that is later fulfilled
info: |
    7. Return PerformPromiseThen(promise, onFulfilled, onRejected,
       resultCapability).

    25.4.5.3.1 PerformPromiseThen

    [...]
    7. If the value of promise's [[PromiseState]] internal slot is "pending",
       a. Append fulfillReaction as the last element of the List that is the
          value of promise's [[PromiseFulfillReactions]] internal slot.
       b. Append rejectReaction as the last element of the List that is the
          value of promise's [[PromiseRejectReactions]] internal slot.
    [...]
flags: [async]
---*/

var value = {};
var resolve;
var p = new Promise(function(_resolve) {
  resolve = _resolve;
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

resolve(value);
