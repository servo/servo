// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-promise.prototype.catch
description: >
  Promise.prototype.catch called with an object-coercible `this` value
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
features: [Symbol]
---*/

var booleanCount = 0;
Boolean.prototype.then = function() {
  booleanCount += 1;
};
Promise.prototype.catch.call(true);
assert.sameValue(booleanCount, 1, 'boolean');

var numberCount = 0;
Number.prototype.then = function() {
  numberCount += 1;
};
Promise.prototype.catch.call(34);
assert.sameValue(numberCount, 1, 'number');

var stringCount = 0;
String.prototype.then = function() {
  stringCount += 1;
};
Promise.prototype.catch.call('');
assert.sameValue(stringCount, 1, 'string');

var symbolCount = 0;
Symbol.prototype.then = function() {
  symbolCount += 1;
};
Promise.prototype.catch.call(Symbol());
assert.sameValue(symbolCount, 1, 'symbol');
