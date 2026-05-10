// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Abrupt completions should not preclude additional jobs
es6id: 25.4.2.1
info: |
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

var promise = new Promise(function(resolve) {
  resolve();
});
var fulfilledCallCount = 0;
var rejectedCallCount = 0;

promise.then(function() {
  fulfilledCallCount += 1;
  throw new Error();
}, function() {
  rejectedCallCount += 1;
});

promise.then(function() {
  if (fulfilledCallCount !== 1) {
    $DONE('Expected "onFulfilled" handler to be invoked exactly once.');
    return;
  }

  if (rejectedCallCount !== 0) {
    $DONE('Expected "onRejected" handler to not be invoked.');
    return;
  }

  $DONE();
}, function() {
  $DONE('This promise should not be rejected.');
});
