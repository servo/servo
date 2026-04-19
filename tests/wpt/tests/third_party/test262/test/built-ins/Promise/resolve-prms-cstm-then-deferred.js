// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Resolving with a resolved Promise instance whose `then` method has been
    overridden after execution of the executor function
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
        [...]
    12. Perform EnqueueJob ("PromiseJobs", PromiseResolveThenableJob,
        «promise, resolution, thenAction»)
flags: [async]
---*/

var returnValue = null;
var value = {};
var resolve;
var thenable = new Promise(function(resolve) {
  resolve();
});
var promise = new Promise(function(_resolve) {
  resolve = _resolve;
});

thenable.then = function(resolve) {
  resolve(value);
};

promise.then(function(val) {
  if (val !== value) {
    $DONE('The promise should be fulfilled with the provided value.');
    return;
  }

  $DONE();
}, function() {
  $DONE('The promise should not be rejected.');
});

returnValue = resolve(thenable);

assert.sameValue(returnValue, undefined, '"resolve" return value');
