// Copyright (C) 2019 Sergey Rubanov. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Invocation of the constructor's `resolve` method for iterable with promise values
esid: sec-promise.any
info: |
  5. Let result be PerformPromiseAny(iteratorRecord, C, promiseCapability).

  Runtime Semantics: PerformPromiseAny

  8. Repeat
    ...
    i. Let nextPromise be ? Call(promiseResolve, constructor, « nextValue »).

flags: [async]
features: [Promise.any, class, arrow-function]
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

Promise.any.call(Custom, values)
  .then(() => {
      assert.sameValue(presolveCallCount, 0, '`Promise.resolve` is never invoked');
      assert.sameValue(cresolveCallCount, 3, '`Custom.resolve` invoked once for every iterated promise');
    }).then($DONE, $DONE);

