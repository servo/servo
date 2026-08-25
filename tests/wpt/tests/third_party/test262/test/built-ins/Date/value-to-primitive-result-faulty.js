// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date-value
description: Exotic `Symbol.toPrimitive` method returns a non-primitive
info: |
  3. If NewTarget is not undefined, then
     a. If Type(value) is Object and value has a [[DateValue]] internal slot, then
        i. Let tv be thisTimeValue(value).
     b. Else,
        i. Let v be ? ToPrimitive(value).

  ToPrimitive ( input [ , PreferredType ] )

  1. If PreferredType was not passed, let hint be "default".
  2. Else if PreferredType is hint String, let hint be "string".
  3. Else PreferredType is hint Number, let hint be "number".
  4. Let exoticToPrim be ? GetMethod(input, @@toPrimitive).
  5. If exoticToPrim is not undefined, then
     a. Let result be ? Call(exoticToPrim, input, « hint »).
     b. If Type(result) is not Object, return result.
     c. Throw a TypeError exception.
features: [Symbol.toPrimitive]
---*/

var faultyToPrimitive = {};
var returnValue;
faultyToPrimitive[Symbol.toPrimitive] = function() {
  return returnValue;
};

returnValue = {};
assert.throws(TypeError, function() {
  new Date(faultyToPrimitive);
}, 'ordinary object');

returnValue = [];
assert.throws(TypeError, function() {
  new Date(faultyToPrimitive);
}, 'Array exotic object');

returnValue = (function() {
  return arguments;
}());
assert.throws(TypeError, function() {
  new Date(faultyToPrimitive);
}, 'arguments exotic object');
