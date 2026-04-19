// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number.prototype.toprecision
description: >
  Return "NaN" if this is NaN
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
  4. If x is NaN, return the String "NaN".
  [...]
---*/

assert.sameValue(
  NaN.toPrecision(undefined),
  "NaN",
  "step 2: If precision is undefined, return ! ToString(x)"
);

var calls = 0;

var p = {
  valueOf: function() {
    calls++;
    return Infinity;
  }
};

assert.sameValue(NaN.toPrecision(p), "NaN", "value");
assert.sameValue(calls, 1, "NaN is checked after ToInteger(precision)");

var n = new Number(NaN);
calls = 0;
assert.sameValue(n.toPrecision(p), "NaN", "object");
assert.sameValue(calls, 1, "Number NaN is checked after ToInteger(precision)");
