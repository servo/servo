// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number-constructor-number-value
description: >
  Return abrupt from ToNumber(value)
info: |
  Number ( value )

  1. If no arguments were passed to this function invocation, let n be +0.
  2. Else, let n be ? ToNumber(value).
  [...]
---*/

var obj1 = {
  valueOf: function() {
    throw new Test262Error();
  }
};

var obj2 = {
  toString: function() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  Number(obj1);
}, "NewTarget is undefined, {}.valueOf");

assert.throws(Test262Error, function() {
  Number(obj2);
}, "NewTarget is undefined, {}.toString");

assert.throws(Test262Error, function() {
  new Number(obj1);
}, "NewTarget is defined, {}.valueOf");

assert.throws(Test262Error, function() {
  new Number(obj2);
}, "NewTarget is defined, {}.toString");
