// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  Error thrown when invoking the instance's `then` method (closing iterator)
esid: sec-performpromiseall
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
        r. Let result be Invoke(nextPromise, "then", «resolveElement,
           resultCapability.[[Reject]]»).
        s. ReturnIfAbrupt(result).
features: [Symbol.iterator]
---*/

var promise = new Promise(function() {});
var returnCount = 0;
var iter = {};
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

Object.defineProperty(promise, "then", {
  value: function() {
    throw new Test262Error();
  }
});

Promise.all(iter);

assert.sameValue(returnCount, 1);
