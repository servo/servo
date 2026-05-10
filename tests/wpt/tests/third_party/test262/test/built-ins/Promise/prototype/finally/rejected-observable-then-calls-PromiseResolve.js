// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-catchfinallyfunctions
description: >
  PromiseResolve() avoids extra Promise capability creation.
info: |
  Catch Finally Functions

  [...]
  7. Let promise be ? PromiseResolve(C, result).
  8. Let thrower be equivalent to a function that throws reason.
  9. Return ? Invoke(promise, "then", « thrower »).

  PromiseResolve ( C, x )

  1. Assert: Type(C) is Object.
  2. If IsPromise(x) is true, then
    a. Let xConstructor be ? Get(x, "constructor").
    b. If SameValue(xConstructor, C) is true, return x.
features: [Promise.prototype.finally]
flags: [async]
---*/

class MyPromise extends Promise {}

var mp1Value = {};
var mp1 = MyPromise.reject(mp1Value);
var mp2 = MyPromise.reject(42);

var thenCalls = [];
var then = Promise.prototype.then;
Promise.prototype.then = function(resolve, reject) {
  thenCalls.push({promise: this, resolve, reject});
  return then.call(this, resolve, reject);
};

mp1.finally(() => mp2).then(value => {
  throw new Test262Error("Expected the promise to be rejected, got resolved with " + value);
}, () => {
  assert.sameValue(thenCalls.length, 5);

  var mp2Calls = thenCalls.filter(c => c.promise === mp2);
  assert.sameValue(mp2Calls.length, 1);
  assert.sameValue(mp2Calls[0].reject, undefined);

  var thrown = false;
  try {
    mp2Calls[0].resolve();
  } catch (error) {
    thrown = true;
    assert.sameValue(error, mp1Value);
  }

  assert(thrown, "Expected resolve() to throw, but it didn't");
}).then($DONE, $DONE);
