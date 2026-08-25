// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Rejected promises ignore resolution after deferred invocation of the
    provided reject function
esid: sec-promise-executor
info: |
    [...]
    9. Let completion be Call(executor, undefined,
       «resolvingFunctions.[[Resolve]], resolvingFunctions.[[Reject]]»).
    10. If completion is an abrupt completion, then
        [...]
    11. Return promise.

    25.4.1.3.2 Promise Resolve Functions

    [...]
    3. Let alreadyResolved be F.[[AlreadyResolved]].
    4. If alreadyResolved.[[Value]] is true, return undefined.
flags: [async]
---*/

var returnValue = null;
var thenable = new Promise(function() {});
var resolve, reject;
var p = new Promise(function(_resolve, _reject) {
  resolve = _resolve;
  reject = _reject;
});

p.then(function() {
  $DONE('The promise should not be fulfilled.');
}, function() {
  $DONE();
});

reject(thenable);
returnValue = resolve();

assert.sameValue(returnValue, undefined, '"resolve" function return value');
