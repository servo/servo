// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-promise.prototype.catch
description: >
  Promise.prototype.catch called with a `this` value whose `then` property is
  an accessor property that returns an abrupt completion
info: |
  1. Let promise be the this value.
  2. Return ? Invoke(promise, "then", «undefined, onRejected»).

  7.3.18 Invoke

  1. Assert: IsPropertyKey(P) is true.
  2. If argumentsList was not passed, let argumentsList be a new empty List.
  3. Let func be ? GetV(V, P).

  7.3.2 GetV

  1. Assert: IsPropertyKey(P) is true.
  2. Let O be ? ToObject(V).
  3. Return ? O.[[Get]](P, V).
---*/

var poisonedThen = Object.defineProperty({}, 'then', {
  get: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  Promise.prototype.catch.call(poisonedThen);
});
