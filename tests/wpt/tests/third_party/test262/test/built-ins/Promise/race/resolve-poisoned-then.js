// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Resolving with an object with a "poisoned" `then` property
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
       a. Return RejectPromise(promise, then.[[value]]).
flags: [async]
---*/

var value = {};
var poisonedThen = Object.defineProperty({}, 'then', {
  get: function() {
    throw value;
  }
});
var thenable = {
  then: function(resolve) {
    resolve(poisonedThen);
  }
};

Promise.race([thenable])
  .then(function() {
    $DONE('The promise should not be fulfilled.');
  }, function(val) {
    if (val !== value) {
      $DONE('The promise should be rejected with the correct value.');
      return;
    }
    $DONE();
  });
