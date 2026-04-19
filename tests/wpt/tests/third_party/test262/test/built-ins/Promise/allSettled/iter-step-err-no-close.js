// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allsettled
description: >
  Error when advancing the provided iterable (not closing iterator)
info: |
  Promise.allSettled ( iterable )

  6. Let result be PerformPromiseAllSettled(iteratorRecord, C, promiseCapability).
  7. If result is an abrupt completion, then
    a. If iteratorRecord.[[Done]] is false, set result to IteratorClose(iteratorRecord, result).
    b. IfAbruptRejectPromise(result, promiseCapability).

  Runtime Semantics: PerformPromiseAllSettled

  6. Repeat
    a. Let next be IteratorStep(iteratorRecord).
    b. If next is an abrupt completion, set iteratorRecord.[[done]] to true.
    c. ReturnIfAbrupt(next).
features: [Promise.allSettled, Symbol.iterator]
---*/

var iterStepThrows = {};
var poisonedDone = {};
var returnCount = 0;
var error = new Test262Error();
Object.defineProperty(poisonedDone, 'done', {
  get() {
    throw error;
  }
});
Object.defineProperty(poisonedDone, 'value', {
  get() {}
});

iterStepThrows[Symbol.iterator] = function() {
  return {
    next() {
      return poisonedDone;
    },
    return() {
      returnCount += 1;
      return {};
    }
  };
};

Promise.allSettled(iterStepThrows);

assert.sameValue(returnCount, 0);
