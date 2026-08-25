// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Use of the value returned by the constructor's `resolve` method.
esid: sec-promise.any
info: |
  Let result be PerformPromiseAny(iteratorRecord, C, promiseCapability).

  PerformPromiseAny

  Repeat
    ...
    i. Let nextPromise be ? Call(promiseResolve, constructor, « nextValue »).
    ...
    r. Perform ? Invoke(nextPromise, "then", « resultCapability.[[Resolve]], rejectElement »).

features: [Promise.any]
---*/

let originalCallCount = 0;
let newCallCount = 0;
let P = function(executor) {
  executor(function() {}, function() {});
};
P.resolve = function() {
  return newThenable;
};

let originalThenable = {
  then() {
    originalCallCount += 1;
  }
};
let newThenable = {
  then() {
    newCallCount += 1;
  }
};

Promise.any.call(P, [originalThenable]);

assert.sameValue(originalCallCount, 0, 'original `then` method not invoked');
assert.sameValue(newCallCount, 1, 'new `then` method invoked exactly once');
