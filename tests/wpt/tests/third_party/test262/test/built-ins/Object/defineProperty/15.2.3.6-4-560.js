// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-560
description: >
    ES5 Attributes - property ([[Get]] is a Function, [[Set]] is a
    Function, [[Enumerable]] is false, [[Configurable]] is false) is
    undeletable
includes: [propertyHelper.js]
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
  enumerable: false,
  configurable: false
});

verifyProperty(obj, "prop", {
  configurable: false,
});
