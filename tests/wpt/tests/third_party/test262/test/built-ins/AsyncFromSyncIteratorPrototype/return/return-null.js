// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%asyncfromsynciteratorprototype%.return
description: >
  If syncIterator's "return" method is `null`,
  a Promise resolved with `undefined` value is returned.
info: |
  %AsyncFromSyncIteratorPrototype%.return ( value )

  [...]
  5. Let return be GetMethod(syncIterator, "return").
  [...]
  7. If return is undefined, then
    a. Let iterResult be ! CreateIterResultObject(value, true).
    b. Perform ! Call(promiseCapability.[[Resolve]], undefined, « iterResult »).
    c. Return promiseCapability.[[Promise]].

  GetMethod ( V, P )

  [...]
  2. Let func be ? GetV(V, P).
  3. If func is either undefined or null, return undefined.
flags: [async]
features: [async-iteration]
includes: [asyncHelpers.js]
---*/

var iterationCount = 0;
var returnGets = 0;

var syncIterator = {
  [Symbol.iterator]() {
    return this;
  },
  next() {
    return {value: 1, done: false};
  },
  get return() {
    returnGets += 1;
    return null;
  },
};

asyncTest(async function() {
  for await (let _ of syncIterator) {
    iterationCount += 1;
    break;
  }

  assert.sameValue(iterationCount, 1);
  assert.sameValue(returnGets, 1);
});
