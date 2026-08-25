// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-isfinite-number
description: >
  Use non-object value returned from @@toPrimitive method
info: |
  isFinite (number)

  1. Let num be ? ToNumber(number).

  ToPrimitive ( input [ , PreferredType ] )

  [...]
  4. Let exoticToPrim be ? GetMethod(input, @@toPrimitive).
  5. If exoticToPrim is not undefined, then
    a. Let result be ? Call(exoticToPrim, input, « hint »).
    b. If Type(result) is not Object, return result.
features: [Symbol.toPrimitive]
---*/

var called = 0;
var obj = {
  valueOf: function() {
    called = NaN;
    return Infinity;
  },
  toString: function() {
    called = NaN;
    return Infinity;
  }
};

obj[Symbol.toPrimitive] = function() {
  called += 1;
  return 42;
};
assert.sameValue(isFinite(obj), true, "use returned value - number");
assert.sameValue(called, 1, "toPrimitive is called - number");

called = 0;
obj[Symbol.toPrimitive] = function() {
  called += 1;
  return "42";
};
assert.sameValue(isFinite(obj), true, "use returned value - string");
assert.sameValue(called, 1, "toPrimitive is called - string");

called = 0;
obj[Symbol.toPrimitive] = function() {
  called += 1;
  return true;
};
assert.sameValue(isFinite(obj), true, "use returned value - boolean true");
assert.sameValue(called, 1, "toPrimitive is called - boolean true");

called = 0;
obj[Symbol.toPrimitive] = function() {
  called += 1;
  return false;
};
assert.sameValue(isFinite(obj), true, "use returned value - boolean false");
assert.sameValue(called, 1, "toPrimitive is called - boolean false");

called = 0;
obj[Symbol.toPrimitive] = function() {
  called += 1;
  return Infinity;
};
assert.sameValue(isFinite(obj), false, "use returned value - Infinity");
assert.sameValue(called, 1, "toPrimitive is called - Infinity");

called = 0;
obj[Symbol.toPrimitive] = function() {
  called += 1;
  return -Infinity;
};
assert.sameValue(isFinite(obj), false, "use returned value - -Infinity");
assert.sameValue(called, 1, "toPrimitive is called - -Infinity");
