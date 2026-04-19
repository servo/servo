// Copyright (c) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date-value
description: >
    Behavior when coercion via `Symbol.toPrimitive` yields a primitive value
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
features: [Symbol.toPrimitive]
---*/

var y = {};
var retVal;

y[Symbol.toPrimitive] = function() {
  return retVal;
};

retVal = 1234;
assert.sameValue(new Date(y).valueOf(), 1234);

retVal = '2016-06-03T19:03:52.872Z';
assert.sameValue(new Date(y).valueOf(), 1464980632872);

retVal = Symbol.toPrimitive;
assert.throws(TypeError, function() {
  new Date(y);
});
