// Copyright (C) 2018 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Throws TypeError when `this` is not an Object
info: |
  RegExp.prototype [ @@matchAll ] ( string )
    1. Let R be the this value.
    2. If Type(R) is not Object, throw a TypeError exception.
features: [Symbol.matchAll]
---*/

var thisValue;
var callMatchAll = function() {
  RegExp.prototype[Symbol.matchAll].call(thisValue, '');
};

thisValue = null;
assert.throws(TypeError, callMatchAll, 'this value is null');

thisValue = true;
assert.throws(TypeError, callMatchAll, 'this value is Boolean');

thisValue = '';
assert.throws(TypeError, callMatchAll, 'this value is String');

thisValue = Symbol();
assert.throws(TypeError, callMatchAll, 'this value is Symbol');

thisValue = 1;
assert.throws(TypeError, callMatchAll, 'this value is Number');
