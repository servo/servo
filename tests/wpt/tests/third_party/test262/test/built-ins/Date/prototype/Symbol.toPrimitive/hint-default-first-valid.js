// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype-@@toprimitive
description: >
    Behavior when `hint` is "default" and first try returns a valid value
info: |
    1. Let O be the this value.
    2. If Type(O) is not Object, throw a TypeError exception.
    3. If hint is the String value "string" or the String value "default", then
       a. Let tryFirst be "string".
    4. Else if hint is the String value "number", then
       a. Let tryFirst be "number".
    5. Else, throw a TypeError exception.
    6. Return OrdinaryToPrimitive(O, tryFirst).
features: [Symbol.toPrimitive]
---*/

var voAccessCount = 0;
var tsCallCount = 0;;
var obj = {
  get valueOf() {
    voAccessCount += 1;
  },
  toString: function() {
    tsCallCount += 1;
    return 'toString test262';
  }
};

assert.sameValue(
  Date.prototype[Symbol.toPrimitive].call(obj, 'default'),
  'toString test262'
);
assert.sameValue(voAccessCount, 0, '`valueOf` method was not referenced');
assert.sameValue(tsCallCount, 1, '`toString` method was invoked exactly once');
