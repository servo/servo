// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%asyncfromsynciteratorprototype%.next
description: next() will reject promise if getter `value` abrupt completes
info: |
  %AsyncFromSyncIteratorPrototype%.next ( value )
  ...
  2. Let promiseCapability be ! NewPromiseCapability(%Promise%).
  ...
  5. Let nextResult be IteratorNext(syncIteratorRecord, value).
  6. IfAbruptRejectPromise(nextResult, promiseCapability).
  7. Let nextDone be IteratorComplete(nextResult).
  8. If AbruptRejectPromise(nextDone, promiseCapability).
  9. Let nextValue be IteratorValue(nextResult).
  10. IfAbruptRejectPromise(nextValue, promiseCapability).
  ...
  18. Return promiseCapability.[[Promise]].

flags: [async]
features: [async-iteration]
---*/

var thrownError = new Error("Catch me.");

var obj = {
  [Symbol.iterator]() {
    return {
      next() {
        return {
          get value() {
            throw thrownError;
          },
          done: false
        }
      }
    };
  }
};

async function* asyncg() {
  yield* obj;
}

asyncg().next().then(
  function (result) {
    throw new Test262Error("Promise should be rejected.");
  },
  function (err) {
    assert.sameValue(err, thrownError, "Promise should be rejected with thrown error");
  }
).then($DONE, $DONE);

