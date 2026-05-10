// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Resolving with a non-thenable object value after invocation of the executor function
es6id: 25.4.3.1
info: |
    [...]
    8. Let resolvingFunctions be CreateResolvingFunctions(promise).
    9. Let completion be Call(executor, undefined,
       «resolvingFunctions.[[Resolve]], resolvingFunctions.[[Reject]]»).

    25.4.1.3.2 Promise Resolve Functions
    [...]
    8. Let then be Get(resolution, "then").
    9. If then is an abrupt completion, then
       [...]
    10. Let thenAction be then.[[value]].
    11. If IsCallable(thenAction) is false, then
        a. Return FulfillPromise(promise, resolution).
flags: [async]
---*/

var returnValue = null;
var nonThenable = {
  then: null
};
var resolve;
var promise = new Promise(function(_resolve) {
  resolve = _resolve;
});

promise.then(function(value) {
  if (value !== nonThenable) {
    $DONE('The promise should be fulfilled with the provided value.');
    return;
  }

  $DONE();
}, function() {
  $DONE('The promise should not be rejected.');
});

returnValue = resolve(nonThenable);

assert.sameValue(returnValue, undefined, '"resolve" return value');
