// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Invocation of the constructor's `resolve` method for iterable with promise values
esid: sec-promise.all
info: |
  Let result be PerformPromiseAll(iteratorRecord, C, promiseCapability).

  Runtime Semantics: PerformPromiseAll

  Repeat
    ...
    i. Let nextPromise be ? Call(promiseResolve, constructor, « nextValue »).

flags: [async]
features: [arrow-function]
---*/

let values = [1,1,1];
let callCount = 0;
let boundPromiseResolve = Promise.resolve.bind(Promise);

Promise.resolve = function(...args) {
  callCount += 1;
  return boundPromiseResolve(...args);
};

Promise.all(values)
  .then(() => {
      assert.sameValue(callCount, 3, '`then` invoked once for every iterated promise');
    }).then($DONE, $DONE);

