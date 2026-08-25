// Copyright (C) 2019 Leo Balter, 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Gets constructor's `resolve` method once from zero to many invocations.
esid: sec-promise.any
info: |
  5. Let result be PerformPromiseAny(iteratorRecord, C, promiseCapability).
  6. If result is an abrupt completion, then
    a. If iteratorRecord.[[Done]] is false, set result to IteratorClose(iteratorRecord, result).
    b. IfAbruptRejectPromise(result, promiseCapability).

  Runtime Semantics: PerformPromiseAny

  6. Let promiseResolve be ? Get(constructor, "resolve").
  7. If ! IsCallable(promiseResolve) is false, throw a TypeError exception.
  8. Repeat
    ...
    i. Let nextPromise be ? Call(promiseResolve, constructor, « nextValue »).

flags: [async]
features: [Promise.any, arrow-function, destructuring-binding]
---*/

let boundPromiseResolve = Promise.resolve.bind(Promise);
let getCount = 0;
let callCount = 0;

Object.defineProperty(Promise, 'resolve', {
  configurable: true,
  get() {
    getCount += 1;
    return function(...args) {
      callCount += 1;
      return boundPromiseResolve(...args);
    };
  }
});

Promise.any([]).then(() => {
    $DONE('The promise should be rejected, but was resolved');
  }, ({errors}) => {
    assert.sameValue(getCount, 1);
    assert.sameValue(callCount, 0);
    assert.sameValue(errors.length, 0);
  }).then($DONE, $DONE);


