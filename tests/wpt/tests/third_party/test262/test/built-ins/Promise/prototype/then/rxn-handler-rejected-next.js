// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: All queued jobs should be executed in series
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
var log = '';

promise.then(function() {
  log += 'A';
}, function() {
  log += 'a';
});

promise.then(function() {
  log += 'B';
}, function() {
  log += 'b';
});

promise.then(function() {
  log += 'C';
}, function() {
  log += 'c';
});

promise.then(function() {
  $DONE('This promise should not be fulfilled.');
}, function() {
  if (log !== 'abc') {
    $DONE(
      'Expected each "onFulfilled" handler to be invoked exactly once in series. ' +
      'Expected: abc. Actual: ' + log
    );
    return;
  }

  $DONE();
});
