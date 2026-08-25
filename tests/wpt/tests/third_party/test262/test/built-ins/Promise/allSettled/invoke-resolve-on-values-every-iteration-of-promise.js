// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Invocation of the constructor's `resolve` method for iterable with non-promise values
esid: sec-promise.allsettled
info: |
  5. Let result be PerformPromiseAllSettled(iteratorRecord, C, promiseCapability).

  Runtime Semantics: PerformPromiseAllSettled

  8. Repeat
    ...
    i. Let nextPromise be ? Call(promiseResolve, constructor, « nextValue »).

flags: [async]
features: [Promise.allSettled, arrow-function]
---*/

let values = [1, 2, 3];
let callCount = 0;
let boundPromiseResolve = Promise.resolve.bind(Promise);

Promise.resolve = function(...args) {
  callCount += 1;
  return boundPromiseResolve(...args);
};

Promise.allSettled(values)
  .then(() => {
      assert.sameValue(callCount, 3, '`Promise.resolve` invoked once for every iterated value');
    }).then($DONE, $DONE);

