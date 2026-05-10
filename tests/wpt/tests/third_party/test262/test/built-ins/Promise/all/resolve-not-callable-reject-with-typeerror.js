// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.all
description: >
  If the constructor's `resolve` method is not callable, reject with a TypeError.
info: |
  Let result be PerformPromiseAny(iteratorRecord, C, promiseCapability).

  Runtime Semantics: PerformPromiseAny

  Let promiseResolve be ? Get(constructor, "resolve").
  If ! IsCallable(promiseResolve) is false, throw a TypeError exception.

flags: [async]
features: [arrow-function]
---*/

Promise.resolve = null;

Promise.all([1])
  .then(
    () => $DONE('The promise should not be resolved.'),
    error => {
      assert(error instanceof TypeError);
    }
  ).then($DONE, $DONE);
