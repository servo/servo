// Copyright (C) 2019 Leo Balter, 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.any
description: >
  Error when advancing the provided iterable (rejecting promise)
info: |
  Promise.any ( iterable )

  5. Let result be PerformPromiseAny(iteratorRecord, C, promiseCapability).
  6. If result is an abrupt completion, then
    a. If iteratorRecord.[[Done]] is false, set result to IteratorClose(iteratorRecord, result).
    b. IfAbruptRejectPromise(result, promiseCapability).

  Runtime Semantics: PerformPromiseAny

  8. Repeat
    a. Let next be IteratorStep(iteratorRecord).
    b. If next is an abrupt completion, set iteratorRecord.[[done]] to true.
    c. ReturnIfAbrupt(next).

flags: [async]
features: [Promise.any, Symbol.iterator, computed-property-names, Symbol, arrow-function]
---*/

let poisonedDone = {};
let error = new Test262Error();
Object.defineProperties(poisonedDone, {
  done: {
    get() {
      throw error;
    }
  },
  value: {
    get() {
      $DONE('The `value` property should not be accessed.');
    }
  }
});
let iterStepThrows = {
  [Symbol.iterator]() {
    return {
      next() {
        return poisonedDone;
      }
    };
  }
};

Promise.any(iterStepThrows).then(
  () => {
  $DONE('The promise should be rejected.');
}, (reason) => {
  assert.sameValue(reason, error);
}).then($DONE, $DONE);
