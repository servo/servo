// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date-value
description: Error retrieving `Symbol.toPrimitive` method from object value
info: |
  3. If NewTarget is not undefined, then
     a. If Type(value) is Object and value has a [[DateValue]] internal slot, then
        i. Let tv be thisTimeValue(value).
     b. Else,
        i. Let v be ? ToPrimitive(value).
        [...]
features: [Symbol.toPrimitive]
---*/

var poisonedObject = {};
var poisonedDate = new Date();
Object.defineProperty(poisonedObject, Symbol.toPrimitive, {
  get: function() {
    throw new Test262Error();
  }
});
Object.defineProperty(poisonedDate, Symbol.toPrimitive, {
  get: function() {
    throw new Test262Error();
  }
});

Date(poisonedObject);

new Date(poisonedDate);

assert.throws(Test262Error, function() {
  new Date(poisonedObject);
});
