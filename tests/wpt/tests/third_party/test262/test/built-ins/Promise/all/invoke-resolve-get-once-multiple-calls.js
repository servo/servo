// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Gets constructor's `resolve` method once from zero to many invocations.
esid: sec-promise.all
info: |
  Runtime Semantics: PerformPromiseAll

  1. Let promiseResolve be ? Get(constructor, `"resolve"`).
  1. If IsCallable(promiseResolve) is false, throw a TypeError exception.
  ...
  1. Repeat,
    ...
    1. Let nextPromise be ? Call(promiseResolve, constructor, &laquo; nextValue &raquo;).
---*/

var p1 = Promise.resolve(1);
var p2 = Promise.resolve(1);
var p3 = Promise.reject(1);
var p4 = Promise.resolve(1);
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

Promise.all([p1, p2, p3, p4]);

assert.sameValue(
  getCount, 1, 'Got `resolve` only once for each iterated value'
);
assert.sameValue(
  callCount, 4, '`resolve` invoked once for each iterated value'
);
