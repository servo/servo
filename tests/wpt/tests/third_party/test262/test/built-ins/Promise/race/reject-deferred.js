// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Rejecting through deferred invocation of the provided resolving function
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

    25.4.1.3.1 Promise Reject Functions
    [...]
    6. Return RejectPromise(promise, reason).
flags: [async]
---*/

var thenable = {
  then: function(_, reject) {
    new Promise(function(resolve) {
        resolve();
      })
      .then(function() {
        reject();
      });
  }
};

Promise.race([thenable])
  .then(function() {
    $DONE('The promise should not be fulfilled.');
  }, function() {
    $DONE();
  });
