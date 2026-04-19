// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-thenfinallyfunctions
description: >
  PromiseResolve() avoids extra Promise capability creation.
info: |
  Then Finally Functions

  [...]
  7. Let promise be ? PromiseResolve(C, result).
  8. Let valueThunk be equivalent to a function that returns value.
  9. Return ? Invoke(promise, "then", « valueThunk »).

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
var mp1 = MyPromise.resolve(mp1Value);
var mp2 = MyPromise.resolve(42);

var thenCalls = [];
var then = Promise.prototype.then;
Promise.prototype.then = function(resolve, reject) {
  thenCalls.push({promise: this, resolve, reject});
  return then.call(this, resolve, reject);
};

mp1.finally(() => mp2).then(() => {
  assert.sameValue(thenCalls.length, 5);

  var mp2Calls = thenCalls.filter(c => c.promise === mp2);
  assert.sameValue(mp2Calls.length, 1);
  assert.sameValue(mp2Calls[0].reject, undefined);
  assert.sameValue(mp2Calls[0].resolve(), mp1Value);
}).then($DONE, $DONE);
