// Copyright (C) 2025 Sosuke Suzuki. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%asynciteratorprototype%-@@asyncDispose
description: rejects if `return` returns rejected promise
info: |
  %AsyncIteratorPrototype% [ @@asyncDispose ] ( )

  ...
  6. Else,
    a. Let result be Call(return, O, « undefined »).
    b. IfAbruptRejectPromise(result, promiseCapability).
    ...

flags: [async]
features: [explicit-resource-management]
includes: [asyncHelpers.js]
---*/

async function* generator() {}
const AsyncIteratorPrototype = Object.getPrototypeOf(Object.getPrototypeOf(generator.prototype));

var returnCount = 0;

function CatchError() {}

const obj = {
  return() {
    returnCount++;
    return Promise.reject(new CatchError());
  }
};

asyncTest(async function () {
  await assert.throwsAsync(CatchError, function () {
    return AsyncIteratorPrototype[Symbol.asyncDispose].call(obj);
  }, "Promise should be rejected");
  assert.sameValue(returnCount, 1);
});
