// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Resolved promises ignore rejections through immediate invocation of the
    provided resolving function
es6id: 25.4.3.1
info: |
    [...]
    9. Let completion be Call(executor, undefined,
       «resolvingFunctions.[[Resolve]], resolvingFunctions.[[Reject]]»).
    10. If completion is an abrupt completion, then
        [...]
    11. Return promise.

    25.4.1.3.1 Promise Reject Functions
    [...]
    3. Let alreadyResolved be the value of F's [[AlreadyResolved]] internal
       slot.
    4. If alreadyResolved.[[value]] is true, return undefined.
flags: [async]
---*/

var returnValue = null;
var thenable = new Promise(function() {});
var p = new Promise(function(resolve, reject) {
  resolve();
  returnValue = reject(thenable);
});

assert.sameValue(returnValue, undefined, '"reject" function return value');

p.then(function() {
  $DONE();
}, function() {
  $DONE('The promise should not be rejected.');
});
