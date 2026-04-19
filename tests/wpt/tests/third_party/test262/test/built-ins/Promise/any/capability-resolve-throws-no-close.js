// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-promise.any
description: >
  Iterator is not closed when the "resolve" capability returns an abrupt
  completion.
info: |
  Let C be the this value.
  Let promiseCapability be ? NewPromiseCapability(C).
  Let iteratorRecord be GetIterator(iterable).
  IfAbruptRejectPromise(iteratorRecord, promiseCapability).
  Let result be PerformPromiseAny(iteratorRecord, C, promiseCapability).
  If result is an abrupt completion, then
    If iteratorRecord.[[Done]] is false, set result to IteratorClose(iteratorRecord, result).
    IfAbruptRejectPromise(result, promiseCapability).
  Return Completion(result).

flags: [async]
features: [Promise.any, Symbol.iterator]
---*/
let callCount = 0;
let nextCount = 0;
let returnCount = 0;
let iter = {
  [Symbol.iterator]() {
    callCount++;
    return {
      next() {
        callCount++;
        nextCount += 1;
        return {
          done: true
        };
      },
      return() {
        callCount++;
        returnCount += 1;
        return {};
      }
    };
  }
};

function P(executor) {
  callCount++;
  return new Promise((_, reject) => {
    callCount++;
    executor(() => {
      callCount++;
      throw new Test262Error();
    }, (...args) => {
      callCount++;
      reject(...args);
    });
  });
};

P.resolve = Promise.resolve;

Promise.any.call(P, iter).then(
  () => {
  $DONE('The promise should be rejected.');
}, (reason) => {
  assert.sameValue(nextCount, 1);
  assert.sameValue(returnCount, 0);
  assert.sameValue(callCount, 5);
}).then($DONE, $DONE);
