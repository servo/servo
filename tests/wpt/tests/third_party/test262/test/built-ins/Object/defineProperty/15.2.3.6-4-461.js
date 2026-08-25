// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-461
description: >
    ES5 Attributes - property ([[Get]] is undefined, [[Set]] is a
    Function, [[Enumerable]] is true, [[Configurable]] is true) is
    deletable
---*/

var obj = {};

var verifySetFunc = "data";
var setFunc = function(value) {
  verifySetFunc = value;
};

Object.defineProperty(obj, "prop", {
  get: undefined,
  set: setFunc,
  enumerable: true,
  configurable: true
});

var propertyDefineCorrect = obj.hasOwnProperty("prop");
var desc = Object.getOwnPropertyDescriptor(obj, "prop");

delete obj.prop;

assert(propertyDefineCorrect, 'propertyDefineCorrect !== true');
assert.sameValue(desc.configurable, true, 'desc.configurable');
assert.sameValue(obj.hasOwnProperty("prop"), false, 'obj.hasOwnProperty("prop")');
