// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-512
description: >
    ES5 Attributes - [[Get]] attribute of accessor property ([[Get]]
    is a Function, [[Set]] is undefined, [[Enumerable]] is false,
    [[Configurable]] is true) is the expected function
---*/

var obj = {};

var getFunc = function() {
  return 1001;
};

Object.defineProperty(obj, "prop", {
  get: getFunc,
  set: undefined,
  enumerable: false,
  configurable: true
});

var propertyDefineCorrect = obj.hasOwnProperty("prop");
var desc = Object.getOwnPropertyDescriptor(obj, "prop");

assert(propertyDefineCorrect, 'propertyDefineCorrect !== true');
assert.sameValue(desc.get, getFunc, 'desc.get');
assert.sameValue(obj.prop, 1001, 'obj.prop');
