// Copyright (c) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date-value
description: >
  Behavior when error thrown by invocation of `Symbol.toPrimitive` method
  during coercion
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
features: [Symbol.toPrimitive]
---*/

var y = {};
y[Symbol.toPrimitive] = function() {
  throw new Test262Error();
};

assert.throws(Test262Error, function() {
  new Date(y);
});
