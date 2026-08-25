// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Resolving with a reference to the promise itself
es6id: 25.4.4.5
info: |
    1. Let C be the this value.
    [...]
    4. Let promiseCapability be NewPromiseCapability(C).
    [...]
    6. Let resolveResult be Call(promiseCapability.[[Resolve]], undefined,
       «x»).
    [...]

    25.4.1.3.2 Promise Resolve Functions
    [...]
    6. If SameValue(resolution, promise) is true, then
       a. Let selfResolutionError be a newly created TypeError object.
       b. Return RejectPromise(promise, selfResolutionError).
flags: [async]
---*/

var resolve, reject;
var promise = new Promise(function(_resolve, _reject) {
  resolve = _resolve;
  reject = _reject;
});
var P = function(executor) {
  executor(resolve, reject);
  return promise;
};

Promise.resolve.call(P, promise)
  .then(function() {
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
