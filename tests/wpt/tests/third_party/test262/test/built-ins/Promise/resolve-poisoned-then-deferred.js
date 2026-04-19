// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Resolving with an object with a "poisoned" `then` property after invocation
    of the executor function
es6id: 25.4.3.1
info: |
    [...]
    8. Let resolvingFunctions be CreateResolvingFunctions(promise).
    9. Let completion be Call(executor, undefined,
       «resolvingFunctions.[[Resolve]], resolvingFunctions.[[Reject]]»).

    25.4.1.3.2 Promise Resolve Functions
    7. If Type(resolution) is not Object, then
       a. Return FulfillPromise(promise, resolution).
flags: [async]
---*/

var returnValue = null;
var value = {};
var resolve;
var poisonedThen = Object.defineProperty({}, 'then', {
  get: function() {
    throw value;
  }
});
var promise = new Promise(function(_resolve) {
  resolve = _resolve;
});

promise.then(function() {
  $DONE('The promise should not be fulfilled.');
}, function(val) {
  if (val !== value) {
    $DONE('The promise should be fulfilled with the provided value.');
    return;
  }

  $DONE();
});

returnValue = resolve(poisonedThen);

assert.sameValue(returnValue, undefined, '"resolve" return value');
