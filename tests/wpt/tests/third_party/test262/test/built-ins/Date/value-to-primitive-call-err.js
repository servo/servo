// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date-value
description: Error invoking `Symbol.toPrimitive` method of object value
info: |
  3. If NewTarget is not undefined, then
     a. If Type(value) is Object and value has a [[DateValue]] internal slot, then
        i. Let tv be thisTimeValue(value).
     b. Else,
        i. Let v be ? ToPrimitive(value).

  ToPrimitive ( input [ , PreferredType ] )

  [...]
  4. Let exoticToPrim be ? GetMethod(input, @@toPrimitive).
features: [Symbol.toPrimitive]
---*/

var badToPrimitive = {};
badToPrimitive[Symbol.toPrimitive] = function() {
  throw new Test262Error();
};

assert.throws(Test262Error, function() {
  new Date(badToPrimitive);
});
