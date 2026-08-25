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
    Let nextPromise be ? Call(promiseResolve, constructor, « nextValue »).

flags: [async]
features: [class, arrow-function]
---*/
class Custom extends Promise {}

let values = [1, 1, 1];
let cresolveCallCount = 0;
let presolveCallCount = 0;
let boundCustomResolve = Custom.resolve.bind(Custom);
let boundPromiseResolve = Promise.resolve.bind(Promise);

Custom.resolve = function(...args) {
  cresolveCallCount += 1;
  return boundCustomResolve(...args);
};

Promise.resolve = function(...args) {
  presolveCallCount += 1;
  return boundPromiseResolve(...args);
};

Promise.all.call(Custom, values)
  .then(() => {
      assert.sameValue(presolveCallCount, 0, '`Promise.resolve` is never invoked');
      assert.sameValue(cresolveCallCount, 3, '`Custom.resolve` invoked once for every iterated promise');
    }).then($DONE, $DONE);

