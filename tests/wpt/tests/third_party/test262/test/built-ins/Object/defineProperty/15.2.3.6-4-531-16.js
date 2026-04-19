// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-531-16
description: >
    ES5 Attributes - Updating an indexed accessor property 'P' using
    simple assignment, 'O' is an Arguments object (8.12.5 step 5.b)
---*/

var obj = (function() {
  return arguments;
}());

var verifySetFunc = "data";
var setFunc = function(value) {
  verifySetFunc = value;
};
var getFunc = function() {
  return verifySetFunc;
};

Object.defineProperty(obj, "0", {
  get: getFunc,
  set: setFunc,
  enumerable: true,
  configurable: true
});

obj[0] = "overrideData";
var propertyDefineCorrect = obj.hasOwnProperty("0");
var desc = Object.getOwnPropertyDescriptor(obj, "0");

assert(propertyDefineCorrect, 'propertyDefineCorrect !== true');
assert.sameValue(desc.set, setFunc, 'desc.set');
assert.sameValue(obj[0], "overrideData", 'obj[0]');
