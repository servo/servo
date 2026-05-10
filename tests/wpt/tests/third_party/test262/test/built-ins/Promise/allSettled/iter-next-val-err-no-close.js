// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allsettled
description: >
  Error when accessing an iterator result's `value` property (not closing
  iterator)
info: |
  Promise.allSettled ( iterable )

  6. Let result be PerformPromiseAllSettled(iteratorRecord, C, promiseCapability).
  7. If result is an abrupt completion, then
    a. If iteratorRecord.[[Done]] is false, set result to IteratorClose(iteratorRecord, result).
    b. IfAbruptRejectPromise(result, promiseCapability).

  Runtime Semantics: PerformPromiseAllSettled

  ...
  6. Repeat
    ...
    e. Let nextValue be IteratorValue(next).
    f. If nextValue is an abrupt completion, set iteratorRecord.[[Done]] to true.
    g. ReturnIfAbrupt(nextValue).
features: [Promise.allSettled, Symbol.iterator]
---*/

var iterNextValThrows = {};
var returnCount = 0;
var nextCount = 0;
var poisonedVal = {
  done: false
};
var error = new Test262Error();
Object.defineProperty(poisonedVal, 'value', {
  get() {
    throw error;
  }
});
iterNextValThrows[Symbol.iterator] = function() {
  return {
    next() {
      nextCount += 1;
      return poisonedVal;
    },
    return() {
      returnCount += 1;
      return {};
    }
  };
};

Promise.allSettled(iterNextValThrows);

assert.sameValue(returnCount, 0);
assert.sameValue(nextCount, 1);
