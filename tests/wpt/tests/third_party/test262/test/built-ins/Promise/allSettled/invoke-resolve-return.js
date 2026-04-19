// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Use of the value returned by the constructor's `resolve` method.
esid: sec-promise.allsettled
info: |
  6. Let result be PerformPromiseAllSettled(iteratorRecord, C, promiseCapability).

  Runtime Semantics: PerformPromiseAllSettled

  6. Repeat
    ...
    i. Let nextPromise be ? Invoke(constructor, "resolve", « nextValue »).
    ...
    z. Perform ? Invoke(nextPromise, "then", « resolveElement, rejectElement »).
features: [Promise.allSettled]
---*/

var originalCallCount = 0;
var newCallCount = 0;
var P = function(executor) {
  executor(function() {}, function() {});
};
P.resolve = function() {
  return newThenable;
};

var originalThenable = {
  then() {
    originalCallCount += 1;
  }
};
var newThenable = {
  then() {
    newCallCount += 1;
  }
};

Promise.allSettled.call(P, [originalThenable]);

assert.sameValue(originalCallCount, 0, 'original `then` method not invoked');
assert.sameValue(newCallCount, 1, 'new `then` method invoked exactly once');
