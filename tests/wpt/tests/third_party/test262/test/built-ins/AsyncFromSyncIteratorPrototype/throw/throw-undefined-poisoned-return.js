// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%asyncfromsynciteratorprototype%.throw
description: throw() will close the iterator and return rejected promise if sync `throw` undefined
info: |
  %AsyncFromSyncIteratorPrototype%.throw ( value )
  ...
  2. Let promiseCapability be ! NewPromiseCapability(%Promise%).
  ...
  5. Let return be GetMethod(syncIterator, "throw").
  6. IfAbruptRejectPromise(throw, promiseCapability).
  7. If throw is undefined, then
    a. NOTE: If syncIterator does not have a throw method, close it to give it a chance to clean up before we reject the capability.
    b. Let closeCompletion be Completion { [[Type]]: normal, [[Value]]: empty, [[Target]]: empty }.
    c. Set result to IteratorClose(syncIteratorRecord, closeCompletion).
    d. IfAbruptRejectPromise(result, promiseCapability).
    e. NOTE: The next step throws a TypeError to indicate that there was a protocol violation: syncIterator does not have a throw method.
    f. NOTE: If closing syncIterator does not throw then the result of that operation is ignored, even if it yields a rejected promise.
    g. Perform ! Call(promiseCapability.[[Reject]], undefined, « a newly created TypeError object »).
    h. Return promiseCapability.[[Promise]].

  IteratorClose ( iterator, completion )
  ...
  2. Let iterator be iteratorRecord.[[Iterator]].
  3. Let innerResult be Completion(GetMethod(iterator, "return")).
  ...
  6. If innerResult.[[Type]] is throw, return ? innerResult.
  ...

  IfAbruptRejectPromise ( value, capability )
  1. Assert: value is a Completion Record.
  2. If value is an abrupt completion, then
    a. Perform ? Call(capability.[[Reject]], undefined, « value.[[Value]] »).
    b. Return capability.[[Promise]].
  ...

flags: [async]
features: [async-iteration]
includes: [asyncHelpers.js]
---*/

var returnCount = 0;
function CatchError() {}
var thrownError = new CatchError();

const obj = {
  [Symbol.iterator]() {
    return {
      next() {
        return {value: 1, done: false};
      },
      get return() {
        returnCount += 1;
        throw thrownError;
      }
    };
  }
};

async function* wrapper() {
  yield* obj;
}

var iter = wrapper();

asyncTest(async function () {
  await assert.throwsAsync(CatchError, async () => {
    await iter.next();
    return iter.throw();
  }, "Promise should be rejected");
  assert.sameValue(returnCount, 1, 'iterator closed properly');
  const result = await iter.next();
  assert(result.done, "the iterator is completed");
  assert.sameValue(result.value, undefined, "value is undefined");
})
