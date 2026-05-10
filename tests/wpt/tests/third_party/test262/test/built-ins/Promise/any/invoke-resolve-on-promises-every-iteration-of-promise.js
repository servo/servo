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
features: [Promise.any, arrow-function]
---*/

let values = [1,1,1];
let callCount = 0;
let boundPromiseResolve = Promise.resolve.bind(Promise);

Promise.resolve = function(...args) {
  callCount += 1;
  return boundPromiseResolve(...args);
};

Promise.any(values)
  .then(() => {
      assert.sameValue(callCount, 3, '`then` invoked once for every iterated promise');
    }).then($DONE, $DONE);

