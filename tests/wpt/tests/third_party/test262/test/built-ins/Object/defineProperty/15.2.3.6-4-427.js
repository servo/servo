// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-427
description: >
    ES5 Attributes - success to update [[Set]] attribute of accessor
    property ([[Get]] is undefined, [[Set]] is undefined,
    [[Enumerable]] is true, [[Configurable]] is true) to different
    value
---*/

var obj = {};
var verifySetFunc = "data";
var setFunc = function(value) {
  verifySetFunc = value;
};

Object.defineProperty(obj, "prop", {
  get: undefined,
  set: undefined,
  enumerable: true,
  configurable: true
});

var desc1 = Object.getOwnPropertyDescriptor(obj, "prop");

Object.defineProperty(obj, "prop", {
  set: setFunc
});

obj.prop = "overrideData";
var propertyDefineCorrect = obj.hasOwnProperty("prop");
var desc2 = Object.getOwnPropertyDescriptor(obj, "prop");

assert(propertyDefineCorrect, 'propertyDefineCorrect !== true');
assert.sameValue(typeof desc1.set, "undefined", 'typeof desc1.set');
assert.sameValue(desc2.set, setFunc, 'desc2.set');
assert.sameValue(verifySetFunc, "overrideData", 'verifySetFunc');
