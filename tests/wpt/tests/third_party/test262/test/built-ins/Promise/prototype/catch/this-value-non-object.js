// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-promise.prototype.catch
description: >
  Promise.prototype.catch called with a non-object-coercible `this` value
info: |
  1. Let promise be the this value.
  2. Return ? Invoke(promise, "then", «undefined, onRejected»).

  7.3.18 Invoke

  1. Assert: IsPropertyKey(P) is true.
  2. If argumentsList was not passed, let argumentsList be a new empty List.
  3. Let func be ? GetV(V, P).
  4. Return ? Call(func, V, argumentsList).

  7.3.2 GetV

  1. Assert: IsPropertyKey(P) is true.
  2. Let O be ? ToObject(V).
---*/

assert.throws(TypeError, function() {
  Promise.prototype.catch.call(undefined);
}, 'undefined');

assert.throws(TypeError, function() {
  Promise.prototype.catch.call(null);
}, 'null');
