// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.all
description: >
  Error when accessing an iterator result's `value` property (not closing
  iterator)
info: |
    11. Let result be PerformPromiseAll(iteratorRecord, C, promiseCapability).
    12. If result is an abrupt completion,
        a. If iteratorRecord.[[done]] is false, let result be
           IteratorClose(iterator, result).
        b. IfAbruptRejectPromise(result, promiseCapability).

    [...]

    25.4.4.1.1 Runtime Semantics: PerformPromiseAll

    [...]
    6. Repeat
        [...]
        e. Let nextValue be IteratorValue(next).
        f. If nextValue is an abrupt completion, set iteratorRecord.[[done]] to
           true.
        g. ReturnIfAbrupt(nextValue).
features: [Symbol.iterator]
---*/

var iterNextValThrows = {};
var returnCount = 0;
var poisonedVal = {
  done: false
};
var error = new Test262Error();
Object.defineProperty(poisonedVal, 'value', {
  get: function() {
    throw error;
  }
});
iterNextValThrows[Symbol.iterator] = function() {
  return {
    next: function() {
      return poisonedVal;
    },
    return: function() {
      returnCount += 1;
      return {};
    }
  };
};

Promise.all(iterNextValThrows);

assert.sameValue(returnCount, 0);
