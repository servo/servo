// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.prototype.hasownproperty
description: >
  ToPropertyKey is performed before ToObject.
info: |
  Object.prototype.hasOwnProperty ( V )

  1. Let P be ? ToPropertyKey(V).
  2. Let O be ? ToObject(this value).

  ToPropertyKey ( argument )

  1. Let key be ? ToPrimitive(argument, hint String).
features: [Symbol.toPrimitive]
---*/

var coercibleKey1 = {
  get toString() {
    this.hint = "string";
    throw new Test262Error();
  },
  get valueOf() {
    this.hint = "defaultOrNumber";
    throw new Test262Error();
  },
};

assert.throws(Test262Error, function() {
  Object.prototype.hasOwnProperty.call(null, coercibleKey1);
});
assert.sameValue(coercibleKey1.hint, "string");


var coercibleKey2 = {};
coercibleKey2[Symbol.toPrimitive] = function(hint) {
  this.hint = hint;
  throw new Test262Error();
};

assert.throws(Test262Error, function() {
  Object.prototype.hasOwnProperty.call(undefined, coercibleKey2);
});
assert.sameValue(coercibleKey2.hint, "string");
