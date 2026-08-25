// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  Error thrown when accessing the instance's `then` method (closing iterator)
esid: sec-promise.race
info: |
    11. Let result be PerformPromiseRace(iteratorRecord, C, promiseCapability).
    12. If result is an abrupt completion,
        a. If iteratorRecord.[[done]] is false, let result be
           IteratorClose(iterator, result).
        b. IfAbruptRejectPromise(result, promiseCapability).

    [...]

    25.4.4.3.1 Runtime Semantics: PerformPromiseRace

    1. Repeat
        [...]
        j. Let result be Invoke(nextPromise, "then",
           «promiseCapability.[[Resolve]], promiseCapability.[[Reject]]»).
        k. ReturnIfAbrupt(result).
features: [Symbol.iterator]
---*/

var promise = new Promise(function() {});
var iter = {};
var returnCount = 0;
iter[Symbol.iterator] = function() {
  return {
    next: function() {
      return {
        done: false,
        value: promise
      };
    },
    return: function() {
      returnCount += 1;
      return {};
    }
  };
};
Object.defineProperty(promise, 'then', {
  get: function() {
    throw new Test262Error();
  }
});

Promise.race(iter);

assert.sameValue(returnCount, 1);
