// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.tojson
description: >
  This value is coerced to primitive with Number hint (exotic @@toPrimitive).
info: |
  Date.prototype.toJSON ( key )

  [...]
  2. Let tv be ? ToPrimitive(O, hint Number).

  ToPrimitive ( input [ , PreferredType ] )

  1. Assert: input is an ECMAScript language value.
  2. If Type(input) is Object, then
    [...]
    d. Let exoticToPrim be ? GetMethod(input, @@toPrimitive).
    e. If exoticToPrim is not undefined, then
      i. Let result be ? Call(exoticToPrim, input, « hint »).
      ii. If Type(result) is not Object, return result.
features: [Symbol, Symbol.toPrimitive]
---*/

var callCount = 0, _this, _arguments;
var result = new Boolean(false);

var obj = {
  toISOString: function() { return result; },
  toString: function() { throw new Test262Error('should not be called'); },
  valueOf: function() { throw new Test262Error('should not be called'); },
};

obj[Symbol.toPrimitive] = function() {
  callCount += 1;
  _this = this;
  _arguments = arguments;
  return 3.14;
};

assert.sameValue(Date.prototype.toJSON.call(obj), result);
assert.sameValue(callCount, 1);
assert.sameValue(_this, obj);
assert.sameValue(_arguments[0], 'number');
assert.sameValue(_arguments.length, 1);
