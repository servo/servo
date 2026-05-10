// Copyright (C) 2025 Sosuke Suzuki. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%asynciteratorprototype%-@@asyncDispose
description: rejects if `return` getter throws
info: |
  %AsyncIteratorPrototype% [ @@asyncDispose ] ( )

  1. Let O be the this value.
  2. Let promiseCapability be ! NewPromiseCapability(%Promise%).
  3. Let return be GetMethod(O, "return").
  4. IfAbruptRejectPromise(return, promiseCapability).
  ...

flags: [async]
features: [explicit-resource-management]
includes: [asyncHelpers.js]
---*/

async function* generator() {}
const AsyncIteratorPrototype = Object.getPrototypeOf(Object.getPrototypeOf(generator.prototype));

var returnGetCount = 0;

function CatchError() {}

const obj = {
  get return() {
    returnGetCount++;
    throw new CatchError();
  }
};

asyncTest(async function () {
  await assert.throwsAsync(CatchError, function () {
    return AsyncIteratorPrototype[Symbol.asyncDispose].call(obj);
  }, "Promise should be rejected");
  assert.sameValue(returnGetCount, 1);
});
