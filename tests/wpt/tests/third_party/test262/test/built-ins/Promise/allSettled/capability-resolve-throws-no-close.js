// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-promise.allsettled
description: >
  Iterator is not closed when the "resolve" capability returns an abrupt
  completion.
info: |
  ...
  3. Let promiseCapability be ? NewPromiseCapability(C).
  ...
  6. Let result be PerformPromiseAllSettled(iteratorRecord, C, promiseCapability).
  7. If result is an abrupt completion, then
    a. If iteratorRecord.[[Done]] is false, set result to IteratorClose(iteratorRecord, result).
    b. IfAbruptRejectPromise(result, promiseCapability).
  8. Return Completion(result).

  Runtime Semantics: PerformPromiseAllSettled

  ...
  6. Repeat
    ...
    d. If next is false, then
      ...
      iii. If remainingElementsCount.[[Value]] is 0, then
        1. Let valuesArray be CreateArrayFromList(values).
        2. Perform ? Call(resultCapability.[[Resolve]], undefined, « valuesArray »).

  IfAbruptRejectPromise

  1. IfAbruptRejectPromise(value, capability).
features: [Promise.allSettled, Symbol.iterator]
---*/

var returnCount = 0;
var iter = {};
iter[Symbol.iterator] = function() {
  return {
    next() {
      return {
        done: true
      };
    },
    return() {
      returnCount += 1;
      return {};
    }
  };
};
var P = function(executor) {
  return new Promise(function(_, reject) {
    executor(function() {
      throw new Test262Error();
    }, reject);
  });
};

P.resolve = function() {
  throw new Test262Error();
};

Promise.allSettled.call(P, iter);

assert.sameValue(returnCount, 0);
