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

var promise = new Promise(function(_, reject) {
  reject();
});
var fulfilledCallCount = 0;
var rejectedCallCount = 0;

promise.then(function() {
  fulfilledCallCount += 1;
}, function() {
  rejectedCallCount += 1;
  throw new Error();
});

promise.then(function() {
  $DONE('This promise should not be fulfilled.');
}, function() {
  if (fulfilledCallCount !== 0) {
    $DONE('Expected "onFulfilled" handler to not be invoked.');
    return;
  }

  if (rejectedCallCount !== 1) {
    $DONE('Expected "onRejected" handler to be invoked exactly once.');
    return;
  }

  $DONE();
});
