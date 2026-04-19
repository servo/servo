// copyright (c) 2016 the v8 project authors. all rights reserved.
// this code is governed by the bsd license found in the license file.
/*---
esid: sec-date-value
description: >
    Behavior when coercion via `Symbol.toPrimitive` yields an Object
info: |
  [...]
  3. If NewTarget is not undefined, then
     a. If Type(value) is Object and value has a [[DateValue]] internal slot,
        then
        [...]
     b. Else,
        i. Let v be ? ToPrimitive(value).

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
  new Date(y);
}, 'ordinary object');

retVal = (function() {
  return arguments;
}());
assert.throws(TypeError, function() {
  new Date(y);
}, 'arguments exotic object');
