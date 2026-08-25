// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Resolving with an object with a "poisoned" `then` property from a pending promise that is later rejected
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
    8. Let then be Get(resolution, "then").
    9. If then is an abrupt completion, then
       a. Return RejectPromise(promise, then.[[value]]).
flags: [async]
---*/

var value = {};
var reject;
var poisonedThen = Object.defineProperty({}, 'then', {
  get: function() {
    throw value;
  }
});
var p1 = new Promise(function(_, _reject) {
  reject = _reject;
});
var p2;

p2 = p1.then(function() {}, function() {
  return poisonedThen;
});

p2.then(function(x) {
  $DONE('The promise should not be fulfilled.');
}, function(x) {
  if (x !== value) {
    $DONE('The promise should be rejected with the thrown exception.');
    return;
  }

  $DONE();
});

reject();
