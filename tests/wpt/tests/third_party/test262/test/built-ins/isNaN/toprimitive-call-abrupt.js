// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-isnan-number
description: >
  Return abrupt completion calling number.@@toPrimitive
info: |
  isNaN (number)

  1. Let num be ? ToNumber(number).

  ToPrimitive ( input [ , PreferredType ] )

  [...]
  4. Let exoticToPrim be ? GetMethod(input, @@toPrimitive).
  5. If exoticToPrim is not undefined, then
    a. Let result be ? Call(exoticToPrim, input, « hint »).
features: [Symbol.toPrimitive]
---*/

var obj = {};
obj[Symbol.toPrimitive] = function() {
  throw new Test262Error();
};

assert.throws(Test262Error, function() {
  isNaN(obj);
});
