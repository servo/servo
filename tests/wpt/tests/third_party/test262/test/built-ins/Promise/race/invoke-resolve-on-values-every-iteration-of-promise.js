// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Invocation of the constructor's `resolve` method for iterable with non-promise values
esid: sec-promise.race
info: |
  Let result be PerformPromiseRace(iteratorRecord, C, promiseCapability, promiseResolve).

  PerformPromiseRace

  Repeat
    ...
    i. Let nextPromise be ? Call(promiseResolve, constructor, « nextValue »).

flags: [async]
features: [arrow-function]
---*/

let values = [1, 2, 3];
let callCount = 0;
let boundPromiseResolve = Promise.resolve.bind(Promise);

Promise.resolve = function(...args) {
  callCount += 1;
  return boundPromiseResolve(...args);
};

Promise.race(values)
  .then(() => {
      assert.sameValue(callCount, 3, '`Promise.resolve` invoked once for every item in iterable arg');
    }).then($DONE, $DONE);

