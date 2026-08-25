// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.prototype.toexponential
description: >
  Throws a TypeError if this value is not a number object or value
info: |
  20.1.3 Properties of the Number Prototype Object

  The Number prototype object is the intrinsic object %NumberPrototype%. The
  Number prototype object is an ordinary object. The Number prototype is itself
  a Number object; it has a [[NumberData]] internal slot with the value +0.

  [...]
  The abstract operation thisNumberValue(value) performs the following steps:

  1. If Type(value) is Number, return value.
  2. If Type(value) is Object and value has a [[NumberData]] internal slot, then
    a. Assert: value's [[NumberData]] internal slot is a Number value.
    b. Return the value of value's [[NumberData]] internal slot.
  3. Throw a TypeError exception.

  Number.prototype.toExponential ( fractionDigits )

  1. Let x be ? thisNumberValue(this value).
  [...]
features: [Symbol]
---*/

var toExponential = Number.prototype.toExponential;

assert.throws(TypeError, function() {
  toExponential.call({}, 1);
}, "{}");

assert.throws(TypeError, function() {
  toExponential.call("1", 1);
}, "string");

assert.throws(TypeError, function() {
  toExponential.call(Number, 1);
}, "Number");

assert.throws(TypeError, function() {
  toExponential.call(true, 1);
}, "true");

assert.throws(TypeError, function() {
  toExponential.call(false, 1);
}, "false");

assert.throws(TypeError, function() {
  toExponential.call(null, 1);
}, "null");

assert.throws(TypeError, function() {
  toExponential.call(undefined, 1);
}, "undefined");

assert.throws(TypeError, function() {
  toExponential.call(Symbol("1"), 1);
}, "symbol");

assert.throws(TypeError, function() {
  toExponential.call([], 1);
}, "[]");
