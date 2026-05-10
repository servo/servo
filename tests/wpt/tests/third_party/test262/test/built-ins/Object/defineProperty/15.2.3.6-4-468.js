// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-468
description: >
    ES5 Attributes - [[Set]] attribute of accessor property ([[Get]]
    is undefined, [[Set]] is a Function, [[Enumerable]] is true,
    [[Configurable]] is false) is the expected function
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
  configurable: false
});

obj.prop = "overrideData";
var propertyDefineCorrect = obj.hasOwnProperty("prop");
var desc = Object.getOwnPropertyDescriptor(obj, "prop");

assert(propertyDefineCorrect, 'propertyDefineCorrect !== true');
assert.sameValue(desc.set, setFunc, 'desc.set');
assert.sameValue(verifySetFunc, "overrideData", 'verifySetFunc');
