// Copyright (C) 2021 Jamie Kyle.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.hasown
description: >
  ToObject is performed before ToPropertyKey.
info: |
  Object.hasOwn ( _O_, _P_ )

  1. Let _obj_ be ? ToObject(_O_).
  2. Let _key_ be ? ToPropertyKey(_P_).

  ToPropertyKey ( argument )

  1. Let key be ? ToPrimitive(argument, hint String).
author: Jamie Kyle
features: [Symbol.toPrimitive, Object.hasOwn]
---*/

var callCount1 = 0;
var coercibleKey1 = {
  get toString() {
    callCount1++;
    throw new Test262Error();
  },
  get valueOf() {
    callCount1++;
    throw new Test262Error();
  },
};

assert.throws(TypeError, function() {
  Object.hasOwn(null, coercibleKey1);
});
assert.sameValue(callCount1, 0, "toString and valueOf must not be called");


var callCount2 = 0;
var coercibleKey2 = {};
coercibleKey2[Symbol.toPrimitive] = function() {
  callCount2++;
  throw new Test262Error();
};

assert.throws(TypeError, function() {
  Object.hasOwn(undefined, coercibleKey2);
});
assert.sameValue(callCount2, 0, "Symbol.toPrimitive must not be called");
