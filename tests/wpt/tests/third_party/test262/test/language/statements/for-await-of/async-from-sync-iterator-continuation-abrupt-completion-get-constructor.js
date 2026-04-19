// Copyright (C) 2019 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncfromsynciteratorcontinuation
description: >
  Reject promise when PromiseResolve in AsyncFromSyncIteratorContinuation throws.
info: |
  25.1.4.4 AsyncFromSyncIteratorContinuation ( result, promiseCapability )
    ...
    5. Let valueWrapper be PromiseResolve(%Promise%, « value »).
    6. IfAbruptRejectPromise(valueWrapper, promiseCapability).
    ...

includes: [compareArray.js]
flags: [async]
features: [async-iteration]
---*/

var expected = [
  "start",

  // `valueWrapper` promise rejected.
  "tick 1",

  // `Await(nextResult)` in 13.7.5.13 done.
  "tick 2",

  // catch handler executed.
  "catch",
];

var actual = [];

async function f() {
  var p = Promise.resolve(0);
  Object.defineProperty(p, "constructor", {
    get() {
      throw new Error();
    }
  });
  actual.push("start");
  for await (var x of [p]);
  actual.push("never reached");
}

Promise.resolve(0)
  .then(() => actual.push("tick 1"))
  .then(() => actual.push("tick 2"))
  .then(() => {
    assert.compareArray(actual, expected);
}).then($DONE, $DONE);

f().catch(() => actual.push("catch"));
