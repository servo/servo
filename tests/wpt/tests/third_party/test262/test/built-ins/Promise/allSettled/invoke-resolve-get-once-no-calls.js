// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Gets constructor's `resolve` method once from zero to many invocations.
esid: sec-promise.allsettled
info: |
  6. Let result be PerformPromiseAllSettled(iteratorRecord, C, promiseCapability).

  Runtime Semantics: PerformPromiseAllSettled

  6. Let promiseResolve be ? Get(constructor, `"resolve"`).
  7. 1. If IsCallable(promiseResolve) is false, throw a TypeError exception.
  8. Repeat
    ...
    i. Let nextPromise be ? Call(promiseResolve, constructor, « nextValue »).
features: [Promise.allSettled]
---*/

var resolve = Promise.resolve;
var getCount = 0;
var callCount = 0;

Object.defineProperty(Promise, 'resolve', {
  configurable: true,
  get() {
    getCount += 1;
    return function() {
      callCount += 1;
      return resolve.apply(Promise, arguments);
    };
  }
});

Promise.allSettled([]);

assert.sameValue(
  getCount, 1, 'Got `resolve` only once for each iterated value'
);
assert.sameValue(
  callCount, 0, '`resolve` not called for empty iterator'
);
