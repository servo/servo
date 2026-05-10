// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date-value
description: Exotic `Symbol.toPrimitive` method returns a String value
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

var stringToPrimitive = {};
stringToPrimitive[Symbol.toPrimitive] = function() {
  return '2016-06-05T18:40:00.000Z';
};

assert.sameValue(new Date(stringToPrimitive).getTime(), 1465152000000);
