// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Invocation of the constructor's `resolve` method for iterable with promise values
esid: sec-promise.allsettled
info: |
  7. Let result be PerformPromiseAllSettled(iteratorRecord, C, promiseCapability).

  Runtime Semantics: PerformPromiseAllSettled

  7. Repeat
    ...
    i. Let nextPromise be ? Call(promiseResolve, constructor, « nextValue »).

flags: [async]
features: [Promise.allSettled, class, arrow-function]
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

Promise.allSettled.call(Custom, values)
  .then(() => {
      assert.sameValue(presolveCallCount, 0, '`Promise.resolve` is never invoked');
      assert.sameValue(cresolveCallCount, 3, '`Custom.resolve` invoked once for every iterated promise');
    }).then($DONE, $DONE);

