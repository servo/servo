// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Resolving with a reference to the promise itself
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
    6. If SameValue(resolution, promise) is true, then
       a. Let selfResolutionError be a newly created TypeError object.
       b. Return RejectPromise(promise, selfResolutionError).
flags: [async]
---*/

var self, resolve;
var builtinResolve = Promise.resolve;
var thenable = {
  then: function(r) {
    resolve = r;
  }
};

try {
  Promise.resolve = function(v) {
    return v;
  };
  self = Promise.race([thenable]);
} finally {
  Promise.resolve = builtinResolve;
}

resolve(self);

self.then(function() {
  $DONE('The promise should not be fulfilled.');
}, function(value) {
  if (!value) {
    $DONE('The promise should be rejected with a value.');
    return;
  }
  if (value.constructor !== TypeError) {
    $DONE('The promise should be rejected with a TypeError instance.');
    return;
  }
  $DONE();
});
