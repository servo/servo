// copyright (c) 2016 the v8 project authors. all rights reserved.
// this code is governed by the bsd license found in the license file.
/*---
esid: sec-date-value
description: Invocation of `Symbol.toPrimitive` method
info: |
  [...]
  3. If NewTarget is not undefined, then
     a. If Type(value) is Object and value has a [[DateValue]] internal slot,
        then
        [...]
     b. Else,
        i. Let v be ? ToPrimitive(value).
        [...]

    ES6 Section 7.1.1 ToPrimitive ( input [, PreferredType] )

    1. If PreferredType was not passed, let hint be "default".
    [...]
    4. Let exoticToPrim be GetMethod(input, @@toPrimitive).
    5. ReturnIfAbrupt(exoticToPrim).
    6. If exoticToPrim is not undefined, then
       a. Let result be Call(exoticToPrim, input, «hint»).
       [...]
features: [Symbol.toPrimitive]
---*/

var y = {};
var callCount = 0;
var thisVal, args;

y[Symbol.toPrimitive] = function() {
  callCount += 1;
  thisVal = this;
  args = arguments;
};

new Date(y);

assert.sameValue(callCount, 1, 'method invoked exactly once');
assert.sameValue(thisVal, y, '`this` value is the object being compared');
assert.sameValue(args.length, 1, 'method invoked with exactly one argument');
assert.sameValue(
  args[0],
  'default',
  'method invoked with the string "default" as the first argument'
);
