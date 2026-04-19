// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Explicit iterator closing in response to error
esid: sec-promise.allsettled
info: |
  6. Let result be PerformPromiseAllSettled(iteratorRecord, C, promiseCapability).
  7. If result is an abrupt completion, then
    a. If iteratorRecord.[[Done]] is false, set result to IteratorClose(iteratorRecord, result).
    b. IfAbruptRejectPromise(result, promiseCapability).

  Runtime Semantics: PerformPromiseAllSettled

  6. Repeat
    ...
    i. Let nextPromise be ? Invoke(constructor, "resolve", « nextValue »).
features: [Promise.allSettled, Symbol.iterator]
---*/

var iterDoneSpy = {};
var callCount = 0;
iterDoneSpy[Symbol.iterator] = function() {
  return {
    next() {
      return {
        value: null,
        done: false
      };
    },
    return() {
      callCount += 1;
    }
  };
};

Promise.resolve = function() {
  throw new Error();
};

Promise.allSettled(iterDoneSpy);

assert.sameValue(callCount, 1);
