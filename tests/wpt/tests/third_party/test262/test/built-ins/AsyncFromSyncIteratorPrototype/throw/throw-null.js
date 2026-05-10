// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%asyncfromsynciteratorprototype%.throw
description: >
  If syncIterator's "throw" method is `null`,
  a Promise rejected with provided value is returned.
info: |
  %AsyncFromSyncIteratorPrototype%.throw ( value )

  [...]
  6. Let throw be GetMethod(syncIterator, "throw").
  [...]
  8. If throw is undefined, then
    ...
    g. Perform ! Call(promiseCapability.[[Reject]], undefined, « a newly created TypeError object »).
    h. Return promiseCapability.[[Promise]].

  GetMethod ( V, P )

  [...]
  2. Let func be ? GetV(V, P).
  3. If func is either undefined or null, return undefined.
flags: [async]
features: [async-iteration]
includes: [asyncHelpers.js]
---*/

var throwGets = 0;
var syncIterator = {
  [Symbol.iterator]() {
    return this;
  },
  next() {
    return {value: 1, done: false};
  },
  get throw() {
    throwGets += 1;
    return null;
  },
};

async function* asyncGenerator() {
  yield* syncIterator;
}

var asyncIterator = asyncGenerator();
var thrownError = { name: "err" };

asyncTest(async function () {
  await assert.throwsAsync(TypeError, async () => {
    await asyncIterator.next();
    return asyncIterator.throw(thrownError);
  }, "Promise should be rejected");
  const result = await asyncIterator.next();
  assert(result.done, "the iterator is completed");
  assert.sameValue(result.value, undefined, "value is undefined");
})
