// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-addition-operator-plus-runtime-semantics-evaluation
es6id: 12.7.3.1
description: >
    Behavior when coercion via `Symbol.toPrimitive` yields an Object
info: |
    [...]
    5. Let lprim be ? ToPrimitive(lval).
    6. Let rprim be ? ToPrimitive(rval).
    [...]

    ES6 Section 7.1.1 ToPrimitive ( input [, PreferredType] )

    [...]
    4. Let exoticToPrim be GetMethod(input, @@toPrimitive).
    5. ReturnIfAbrupt(exoticToPrim).
    6. If exoticToPrim is not undefined, then
       a. Let result be Call(exoticToPrim, input, «hint»).
       b. ReturnIfAbrupt(result).
       c. If Type(result) is not Object, return result.
       d. Throw a TypeError exception.
features: [Symbol.toPrimitive]
---*/

var y = {};
var retVal;

y[Symbol.toPrimitive] = function() {
  return retVal;
};

retVal = {};
assert.throws(TypeError, function() {
  0 + y;
}, 'ordinary object value, right-hand side');
assert.throws(TypeError, function() {
  y + 0;
}, 'ordinary object value, left-hand side');

retVal = (function() { return arguments; }());
assert.throws(TypeError, function() {
  0 + y;
}, 'arguments exotic object value, right-hand side');
assert.throws(TypeError, function() {
  y + 0;
}, 'arguments exotic object value, left-hand side');
