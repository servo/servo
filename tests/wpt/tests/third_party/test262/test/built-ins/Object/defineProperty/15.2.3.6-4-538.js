// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-538
description: >
    ES5 Attributes - success to update the accessor property ([[Get]]
    is a Function, [[Set]] is a Function, [[Enumerable]] is true,
    [[Configurable]] is true) to a data property
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
  enumerable: true,
  configurable: true
});
var desc1 = Object.getOwnPropertyDescriptor(obj, "prop");

Object.defineProperty(obj, "prop", {
  value: 1001
});
var desc2 = Object.getOwnPropertyDescriptor(obj, "prop");

if (!desc1.hasOwnProperty("get")) {
  throw new Test262Error('Expected desc1.hasOwnProperty("get") to be true, actually ' + desc1.hasOwnProperty("get"));
}

if (!desc2.hasOwnProperty("value")) {
  throw new Test262Error('Expected desc2.hasOwnProperty("value") to be true, actually ' + desc2.hasOwnProperty("value"));
}

if (typeof desc2.get !== "undefined") {
  throw new Test262Error('Expected typeof desc2.get === "undefined" , actually ' + typeof desc2.get);
}


verifyEqualTo(obj, "prop", 1001);

verifyNotWritable(obj, "prop");

verifyProperty(obj, "prop", {
  enumerable: true,
  configurable: true,
});
