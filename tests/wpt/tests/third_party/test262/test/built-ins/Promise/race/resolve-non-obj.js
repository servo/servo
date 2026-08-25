// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Resolving with a non-object value
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
    7. If Type(resolution) is not Object, then
       a. Return FulfillPromise(promise, resolution).
flags: [async]
---*/

var thenable = {
  then: function(resolve) {
    resolve(23);
  }
};

Promise.race([thenable])
  .then(function(value) {
    if (value !== 23) {
      $DONE('The promise should be resolved with the correct value.');
      return;
    }
    $DONE();
  }, function() {
    $DONE('The promise should not be rejected.');
  });
