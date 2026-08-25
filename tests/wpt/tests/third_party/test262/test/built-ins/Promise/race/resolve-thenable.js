// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Resolving with a thenable object value
es6id: 25.4.4.3
info: |
    [...]
    6. Let promiseCapability be NewPromiseCapability(C).
    [...]
    11. Let result be PerformPromiseRace(iteratorRecord, promiseCapability, C).
    [...]

    25.4.4.3.1 Runtime Semantics: PerformPromiseRace
    1. Repeat
       [...]
       j. Let result be Invoke(nextPromise, "then",
          «promiseCapability.[[Resolve]], promiseCapability.[[Reject]]»).

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

var value = {};
var thenableValue = {
  then: function(resolve) {
    resolve(value);
  }
};
var thenable = {
  then: function(resolve) {
    resolve(thenableValue);
  }
};

Promise.race([thenable])
  .then(function(val) {
    if (val !== value) {
      $DONE('The promise should be resolved with the correct value.');
      return;
    }
    $DONE();
  }, function() {
    $DONE('The promise should not be rejected.');
  });
