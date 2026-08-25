// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%asyncfromsynciteratorprototype%.throw
description: throw() will reject promise if resolving result promise abrupt completes.
info: |
  %AsyncFromSyncIteratorPrototype%.throw ( value )
  ...
  3. Let promiseCapability be ! NewPromiseCapability(%Promise%).
  4. Let syncIteratorRecord be O.[[SyncIteratorRecord]].
  5. Let syncIterator be syncIteratorRecord.[[Iterator]].
  6. Let throw be GetMethod(syncIterator, "throw").
  ...
  9. If value is present, then
    ...
  10. Else,
    a. Let result be Call(throw,syncIterator).
  ...
  13. Return ! AsyncFromSyncIteratorContinuation(result, promiseCapability, syncIteratorRecord, true).

  AsyncFromSyncIteratorContinuation ( result, promiseCapability, syncIteratorRecord, closeOnRejection )
  1. Let done be IteratorComplete(result).
  2. IfAbruptRejectPromise(done, promiseCapability).
  3. Let value be IteratorValue(result).
  4. IfAbruptRejectPromise(value, promiseCapability).
  5. Let valueWrapper be PromiseResolve(%Promise%, value).
  6. If valueWrapper is an abrupt completion, done is false, and closeOnRejection is true, then
    a. Set valueWrapper to IteratorClose(syncIteratorRecord, valueWrapper).
  ...
  12. If done is true, or if closeOnRejection is false, then
    ...
  13. Else,
    a. Let closeIterator be a new Abstract Closure with parameters (error) that captures syncIteratorRecord and performs the following steps when called:
        i. Return ? IteratorClose(syncIteratorRecord, ThrowCompletion(error)).
    b. Let onRejected be CreateBuiltinFunction(closeIterator, 1, "", « »).
    c. NOTE: onRejected is used to close the Iterator when the "value" property of an IteratorResult object it yields is a rejected promise.
  14. Perform PerformPromiseThen(valueWrapper, onFulfilled, onRejected, promiseCapability).
  15. Return promiseCapability.[[Promise]].

flags: [async]
features: [async-iteration]
includes: [asyncHelpers.js]
---*/

var returnCount = 0;
function Reject() {}

var obj = {
  [Symbol.iterator]() {
    return {
      next() {
        return { value: 1, done: false };
      },
      throw() {
        return {
          value: Promise.reject(new Reject()),
          done: false
        };
      },
      return() {
        returnCount += 1;
      }
    };
  }
};

async function* asyncg() {
  return yield* obj;
}

let iter = asyncg();

asyncTest(async function () {
  await assert.throwsAsync(Reject, async () => {
    await iter.next();
    return iter.throw();
  }, "Promise should be rejected");
  assert.sameValue(returnCount, 1);
  const result = await iter.next();
  assert(result.done, "the iterator is completed");
  assert.sameValue(result.value, undefined, "value is undefined");
})
