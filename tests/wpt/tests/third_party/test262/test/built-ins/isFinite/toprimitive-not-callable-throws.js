// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-isfinite-number
description: >
  Throws a TypeError if number.@@toPrimitive is not null, undefined, or callable
info: |
  isFinite (number)

  1. Let num be ? ToNumber(number).

  ToPrimitive ( input [ , PreferredType ] )

  [...]
  4. Let exoticToPrim be ? GetMethod(input, @@toPrimitive).

  GetMethod (V, P)

  [...]
  2. Let func be ? GetV(V, P).
  3. If func is either undefined or null, return undefined.
  4. If IsCallable(func) is false, throw a TypeError exception.
features: [Symbol.toPrimitive]
---*/

var obj = {};

obj[Symbol.toPrimitive] = 42;
assert.throws(TypeError, function() {
  isFinite(obj);
}, "number");

obj[Symbol.toPrimitive] = "";
assert.throws(TypeError, function() {
  isFinite(obj);
}, "string");

obj[Symbol.toPrimitive] = true;
assert.throws(TypeError, function() {
  isFinite(obj);
}, "boolean");

obj[Symbol.toPrimitive] = Symbol.toPrimitive;
assert.throws(TypeError, function() {
  isFinite(obj);
}, "symbol");

obj[Symbol.toPrimitive] = {};
assert.throws(TypeError, function() {
  isFinite(obj);
}, "object");
