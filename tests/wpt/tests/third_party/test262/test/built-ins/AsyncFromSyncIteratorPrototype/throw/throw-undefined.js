// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%asyncfromsynciteratorprototype%.throw
description: throw() will return rejected promise if sync `throw` undefined
info: |
  %AsyncFromSyncIteratorPrototype%.throw ( value )
  ...
  2. Let promiseCapability be ! NewPromiseCapability(%Promise%).
  ...
  6. Let throw be GetMethod(syncIterator, "throw").
  [...]
  8. If throw is undefined, then
    ...
    g. Perform ! Call(promiseCapability.[[Reject]], undefined, « a newly created TypeError object »).
    h. Return promiseCapability.[[Promise]].

flags: [async]
features: [async-iteration]
includes: [asyncHelpers.js]
---*/

var obj = {
  [Symbol.iterator]() {
    return {
      next() {
        return { value: 1, done: false };
      }
    };
  }
};

async function* asyncg() {
  yield* obj;
}

var iter = asyncg();

asyncTest(async function () {
  await assert.throwsAsync(TypeError, async () => {
    await iter.next();
    return iter.throw();
  }, "Promise should be rejected");
  const result = await iter.next();
  assert(result.done, "the iterator is completed");
  assert.sameValue(result.value, undefined, "value is undefined");
})
