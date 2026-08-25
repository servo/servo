// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.prototype.toprecision
description: >
  Return abrupt completion from ToInteger(precision)
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

  Number.prototype.toPrecision ( precision )

  1. Let x be ? thisNumberValue(this value).
  2. If precision is undefined, return ! ToString(x).
  3. Let p be ? ToInteger(precision).
  [...]
---*/

var p1 = {
  valueOf: function() {
    throw new Test262Error();
  }
};

var p2 = {
  toString: function() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  Number.prototype.toPrecision(p1);
}, "valueOf");

assert.throws(Test262Error, function() {
  Number.prototype.toPrecision(p2);
}, "toString");
