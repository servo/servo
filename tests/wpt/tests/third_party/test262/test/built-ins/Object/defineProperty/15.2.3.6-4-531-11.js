// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-531-11
description: >
    Object.defineProperty will update [[Get]] and [[Set]] attributes
    of named accessor property 'P' successfully when [[Configurable]]
    attribute is true, 'A' is an Array object (8.12.9 step 11)
includes: [propertyHelper.js]
---*/


var obj = [];

obj.verifySetFunction = "data";
Object.defineProperty(obj, "prop", {
  get: function() {
    return obj.verifySetFunction;
  },
  set: function(value) {
    obj.verifySetFunction = value;
  },
  configurable: true
});

obj.verifySetFunction1 = "data1";
var getFunc = function() {
  return obj.verifySetFunction1;
};
var setFunc = function(value) {
  obj.verifySetFunction1 = value;
};

Object.defineProperty(obj, "prop", {
  get: getFunc,
  set: setFunc
});

verifyEqualTo(obj, "prop", getFunc());

verifyWritable(obj, "prop", "verifySetFunction1");

verifyProperty(obj, "prop", {
  enumerable: false,
  configurable: true,
});
