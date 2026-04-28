// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.tojson
description: >
  This value is coerced to primitive with Number hint (OrdinaryToPrimitive).
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
---*/

var callCount = 0, _this, _arguments;
var result = [];
var obj = {
  toISOString: function() { return result; },
  toString: function() { throw new Test262Error('should not be called'); },
  valueOf: function() {
    callCount += 1;
    _this = this;
    _arguments = arguments;
    return 'NaN';
  },
};

assert.sameValue(Date.prototype.toJSON.call(obj), result);
assert.sameValue(callCount, 1);
assert.sameValue(_this, obj);
assert.sameValue(_arguments.length, 0);
