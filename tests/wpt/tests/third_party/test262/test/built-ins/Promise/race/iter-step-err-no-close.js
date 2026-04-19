// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  Error when advancing the provided iterable (not closing iterator)
esid: sec-promise.race
info: |
    [...]
    11. Let result be PerformPromiseRace(iteratorRecord, promiseCapability, C).
    12. If result is an abrupt completion, then
        a. If iteratorRecord.[[done]] is false, let result be
           IteratorClose(iterator,result).
        b. IfAbruptRejectPromise(result, promiseCapability).

    25.4.4.3.1 Runtime Semantics: PerformPromiseRace
    1. Repeat
       a. Let next be IteratorStep(iteratorRecord.[[iterator]]).
       b. If next is an abrupt completion, set iteratorRecord.[[done]] to true.
       c. ReturnIfAbrupt(next).
features: [Symbol.iterator]
---*/

var iterStepThrows = {};
var poisonedDone = {};
var returnCount = 0;
var error = new Test262Error();
Object.defineProperty(poisonedDone, 'done', {
  get: function() {
    throw error;
  }
});
Object.defineProperty(poisonedDone, 'value', {
  get: function() {
    throw new Test262Error('The `value` property should not be accessed.');
  }
});

iterStepThrows[Symbol.iterator] = function() {
  return {
    next: function() {
      return poisonedDone;
    },
    return: function() {
      returnCount += 1;
    }
  };
};

Promise.race(iterStepThrows);

assert.sameValue(returnCount, 0);
