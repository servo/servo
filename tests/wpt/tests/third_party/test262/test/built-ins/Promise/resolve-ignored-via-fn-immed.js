// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Rejected promises ignore resolution after immediate invocation of the
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
var p = new Promise(function(resolve, reject) {
  reject(thenable);
  returnValue = resolve();
});

assert.sameValue(returnValue, undefined, '"reject" function return value');

p.then(function() {
  $DONE('The promise should not be fulfilled.');
}, function() {
  $DONE();
});
