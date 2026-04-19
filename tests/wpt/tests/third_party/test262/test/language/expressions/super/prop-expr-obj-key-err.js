// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-super-keyword
es6id: 12.3.5
description: Abrupt completion from type coercion of property key
info: |
  1. Let propertyNameReference be the result of evaluating Expression.
  2. Let propertyNameValue be ? GetValue(propertyNameReference).
  3. Let propertyKey be ? ToPropertyKey(propertyNameValue).

  7.1.14 ToPropertyKey

  1. Let key be ? ToPrimitive(argument, hint String).
---*/

var thrown = new Test262Error();
var badToString = {
  toString: function() {
    throw thrown;
  }
};
var caught;
var obj = {
  method() {
    try {
      super[badToString];
    } catch (err) {
      caught = err;
    }
  }
};

obj.method();

assert.sameValue(caught, thrown);
