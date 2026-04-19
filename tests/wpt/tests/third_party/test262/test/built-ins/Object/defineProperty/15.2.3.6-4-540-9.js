// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-540-9
description: >
    ES5 Attributes - Updating a named accessor property 'P' using
    simple assignment is successful, 'A' is an Array object (8.12.5
    step 5.b)
---*/

var obj = [];

obj.verifySetFunc = "data";
var getFunc = function() {
  return obj.verifySetFunc;
};

var setFunc = function(value) {
  obj.verifySetFunc = value;
};

Object.defineProperty(obj, "prop", {
  get: getFunc,
  set: setFunc,
  enumerable: true,
  configurable: false
});

obj.prop = "overrideData";
var propertyDefineCorrect = obj.hasOwnProperty("prop");
var desc = Object.getOwnPropertyDescriptor(obj, "prop");

assert(propertyDefineCorrect, 'propertyDefineCorrect !== true');
assert.sameValue(desc.set, setFunc, 'desc.set');
assert.sameValue(obj.verifySetFunc, "overrideData", 'obj.verifySetFunc');
