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

features: [Promise.any, Symbol.iterator]
flags: [async]
---*/
let callCount = 0;
let thrown = new Test262Error();
function P(executor) {
  callCount++;
  return new Promise((_, reject) => {
    callCount++;
    executor(() => {
      callCount++;
      throw thrown;
    }, (...args) => {
      callCount++;
      reject(...args);
    });
  });
};
P.resolve = Promise.resolve;

Promise.any.call(P, [1])
  .then(() => {
    $DONE('Promise incorrectly fulfilled.');
  }, (error) => {
    // The error was not the result of promise
    // resolution, so will not be an AggregateError
    assert.sameValue(thrown, error);
    assert.sameValue(callCount, 6);
  }).then($DONE, $DONE);


