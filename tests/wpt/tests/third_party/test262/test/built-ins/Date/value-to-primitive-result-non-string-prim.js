// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date-value
description: >
  Exotic `Symbol.toPrimitive` method returns a primitive value other than a
  string
info: |
  3. If NewTarget is not undefined, then
     a. If Type(value) is Object and value has a [[DateValue]] internal slot, then
        i. Let tv be thisTimeValue(value).
     b. Else,
        i. Let v be ? ToPrimitive(value).
        ii. If Type(v) is String, then
            1. Let tv be the result of parsing v as a date, in exactly the same
               manner as for the parse method (20.3.3.2). If the parse resulted
               in an abrupt completion, tv is the Completion Record.

  ToPrimitive ( input [ , PreferredType ] )

  1. If PreferredType was not passed, let hint be "default".
  2. Else if PreferredType is hint String, let hint be "string".
  3. Else PreferredType is hint Number, let hint be "number".
  4. Let exoticToPrim be ? GetMethod(input, @@toPrimitive).
  5. If exoticToPrim is not undefined, then
     a. Let result be ? Call(exoticToPrim, input, « hint »).
     b. If Type(result) is not Object, return result.
features: [Symbol.toPrimitive]
---*/

var toPrimitive = {};
var returnValue;
toPrimitive[Symbol.toPrimitive] = function() {
  return returnValue;
};

returnValue = 8;
assert.sameValue(new Date(toPrimitive).getTime(), 8, 'number');

returnValue = undefined;
assert.sameValue(new Date(toPrimitive).getTime(), NaN, 'undefined');

returnValue = true;
assert.sameValue(new Date(toPrimitive).getTime(), 1, 'boolean `true`');

returnValue = false;
assert.sameValue(new Date(toPrimitive).getTime(), 0, 'boolean `false`');

returnValue = null;
assert.sameValue(new Date(toPrimitive).getTime(), 0, 'null');

returnValue = Symbol.toPrimitive;
assert.throws(TypeError, function() {
  new Date(toPrimitive);
}, 'symbol');
