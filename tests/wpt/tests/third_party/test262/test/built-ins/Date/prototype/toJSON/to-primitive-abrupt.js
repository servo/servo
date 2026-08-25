// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.tojson
description: >
  Abrupt completion from ToPrimitive.
info: |
  Date.prototype.toJSON ( key )

  [...]
  2. Let tv be ? ToPrimitive(O, hint Number).

  ToPrimitive ( input [ , PreferredType ] )

  1. Assert: input is an ECMAScript language value.
  2. If Type(input) is Object, then
    [...]
    g. Return ? OrdinaryToPrimitive(input, hint).

  OrdinaryToPrimitive ( O, hint )

  [...]
  5. For each name in methodNames in List order, do
    a. Let method be ? Get(O, name).
    b. If IsCallable(method) is true, then
      i. Let result be ? Call(method, O).
      ii. If Type(result) is not Object, return result.
  6. Throw a TypeError exception.
---*/

var toJSON = Date.prototype.toJSON;
var getAbrupt = {
  get valueOf() {
    throw new Test262Error();
  },
};

assert.throws(Test262Error, function() {
  toJSON.call(getAbrupt);
});

var callAbrupt = {
  toString: function() {
    throw new Test262Error();
  },
};

assert.throws(Test262Error, function() {
  toJSON.call(callAbrupt);
});

var notCoercible = Object.create(null);

assert.throws(TypeError, function() {
  toJSON.call(notCoercible);
});
