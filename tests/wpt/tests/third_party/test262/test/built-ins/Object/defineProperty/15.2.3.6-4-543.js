// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-543
description: >
    ES5 Attributes - fail to update [[Get]] attribute of accessor
    property ([[Get]] is a Function, [[Set]] is a Function,
    [[Enumerable]] is true, [[Configurable]] is false) to different
    value
---*/

var obj = {};

var getFunc = function() {
  return 1001;
};

var verifySetFunc = "data";
var setFunc = function(value) {
  verifySetFunc = value;
};

Object.defineProperty(obj, "prop", {
  get: getFunc,
  set: setFunc,
  enumerable: true,
  configurable: false
});

var result1 = obj.prop === 1001;
var desc1 = Object.getOwnPropertyDescriptor(obj, "prop");
assert.throws(TypeError, function() {
  Object.defineProperty(obj, "prop", {
    get: undefined
  });
});
var result2 = obj.prop === 1001;
var desc2 = Object.getOwnPropertyDescriptor(obj, "prop");

assert(result1, 'result1 !== true');
assert(result2, 'result2 !== true');
assert.sameValue(desc1.get, getFunc, 'desc1.get');
assert.sameValue(desc2.get, getFunc, 'desc2.get');
