// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-458
description: >
    ES5 Attributes - [[Get]] attribute of accessor property ([[Get]]
    is undefined, [[Set]] is a Function, [[Enumerable]] is true,
    [[Configurable]] is true) is undefined
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

assert(propertyDefineCorrect, 'propertyDefineCorrect !== true');
assert.sameValue(typeof desc.get, "undefined", 'typeof desc.get');
