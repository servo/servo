// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Rejecting through an abrupt completion
es6id: 25.4.3.1
info: |
    [...]
    9. Let completion be Call(executor, undefined,
       «resolvingFunctions.[[Resolve]], resolvingFunctions.[[Reject]]»).
    10. If completion is an abrupt completion, then
        a. Let status be Call(resolvingFunctions.[[Reject]], undefined,
           «completion.[[value]]»).
        b. ReturnIfAbrupt(status).
    11. Return promise.

    25.4.1.3.1 Promise Reject Functions
    [...]
    6. Return RejectPromise(promise, reason).
flags: [async]
---*/

var thenable = new Promise(function() {});
var p = new Promise(function() {
  throw thenable;
});

p.then(function() {
  $DONE('The promise should not be fulfilled.');
}, function(x) {
  if (x !== thenable) {
    $DONE('The promise should be rejected with the resolution value.');
    return;
  }

  $DONE();
});
