// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-531-15
description: >
    ES5 Attributes - Updating a named accessor property 'P' using
    simple assignment is successful, 'A' is an Array object (8.12.5
    step 5.b)
---*/

var obj = [];

var verifySetFunc = "data";
var setFunc = function(value) {
  verifySetFunc = value;
};
var getFunc = function() {
  return verifySetFunc;
};

Object.defineProperty(obj, "prop", {
  get: getFunc,
  set: setFunc,
  enumerable: true,
  configurable: true
});

obj.prop = "overrideData";
var propertyDefineCorrect = obj.hasOwnProperty("prop");
var desc = Object.getOwnPropertyDescriptor(obj, "prop");

assert(propertyDefineCorrect, 'propertyDefineCorrect !== true');
assert.sameValue(desc.set, setFunc, 'desc.set');
assert.sameValue(obj.prop, "overrideData", 'obj.prop');
