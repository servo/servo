// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.any
description: >
  Error when accessing an iterator result's `value` property (rejecting
  promise)
info: |
  Let result be PerformPromiseAny(iteratorRecord, C, promiseCapability).
  If result is an abrupt completion, then
    If iteratorRecord.[[Done]] is false, set result to IteratorClose(iteratorRecord, result).
    IfAbruptRejectPromise(result, promiseCapability).

  ...

  Runtime Semantics: PerformPromiseAny

  ...
  Repeat
    Let nextValue be IteratorValue(next).
    If nextValue is an abrupt completion, set iteratorRecord.[[Done]] to true.
    ReturnIfAbrupt(nextValue).

features: [Promise.any, Symbol.iterator]
flags: [async]
---*/
let callCount = 0;
let error = new Test262Error();
let poisoned = {
  done: false
};
Object.defineProperty(poisoned, 'value', {
  get() {
    callCount++;
    throw error;
  }
});
let iterNextValThrows = {
  [Symbol.iterator]() {
    callCount++;
    return {
      next() {
        callCount++;
        return poisoned;
      }
    };
  }
};

Promise.any(iterNextValThrows).then(() => {
  $DONE('The promise should be rejected, but was resolved');
}, (reason) => {
  assert(error instanceof Test262Error);
  assert.sameValue(reason, error);
  assert.sameValue(callCount, 3, 'callCount === 3');
}).then($DONE, $DONE);
