// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-531-3
description: >
    Object.defineProperty will update [[Get]] and [[Set]] attributes
    of named accessor property successfully when [[Configurable]]
    attribute is true, 'O' is an Arguments object (8.12.9 step 11)
includes: [propertyHelper.js]
---*/


var obj = (function() {
  return arguments;
}());

obj.verifySetFunction = "data";
Object.defineProperty(obj, "property", {
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

Object.defineProperty(obj, "property", {
  get: getFunc,
  set: setFunc
});

verifyEqualTo(obj, "property", getFunc());

verifyWritable(obj, "property", "verifySetFunction1");

verifyProperty(obj, "property", {
  enumerable: false,
  configurable: true,
});
