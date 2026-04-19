// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Gets constructor's `resolve` method once from zero to many invocations.
esid: sec-promise.race
info: |
  Runtime Semantics: PerformPromiseRace

  1. Let promiseResolve be ? Get(constructor, `"resolve"`).
  1. If IsCallable(promiseResolve) is false, throw a TypeError exception.
  ...
  1. Repeat,
    ...
    1. Let nextPromise be ? Call(promiseResolve, constructor, &laquo; nextValue &raquo;).
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

Promise.race([]);

assert.sameValue(
  getCount, 1, 'Got `resolve` only once for each iterated value'
);
assert.sameValue(
  callCount, 0, '`resolve` not called for empty iterator'
);
