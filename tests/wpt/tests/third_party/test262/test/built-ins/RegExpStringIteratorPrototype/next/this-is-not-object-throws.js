// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Throws TypeError when `this` is not an Object
info: |
  %RegExpStringIteratorPrototype%.next ( )
    1. Let O be the this value.
    2. If Type(O) is not Object, throw a TypeError exception.
features: [Symbol.matchAll]
---*/

var RegExpStringIteratorProto = Object.getPrototypeOf(/./[Symbol.matchAll](''));

var thisValue;
var callNext = function() {
  RegExpStringIteratorProto.next.call(thisValue);
};

thisValue = null;
assert.throws(TypeError, callNext, 'this value is null');

thisValue = true;
assert.throws(TypeError, callNext, 'this value is Boolean');

thisValue = '';
assert.throws(TypeError, callNext, 'this value is String');

thisValue = Symbol();
assert.throws(TypeError, callNext, 'this value is Symbol');

thisValue = 1;
assert.throws(TypeError, callNext, 'this value is Number');

