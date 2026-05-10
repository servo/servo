// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-492
description: >
    ES5 Attributes - fail to update [[Configurable]] attribute of
    accessor property ([[Get]] is undefined, [[Set]] is a Function,
    [[Enumerable]] is false, [[Configurable]] is false) to different
    value
includes: [propertyHelper.js]
---*/

var obj = {};

var verifySetFunc = "data";
var setFunc = function(value) {
  verifySetFunc = value;
};

Object.defineProperty(obj, "prop", {
  get: undefined,
  set: setFunc,
  enumerable: false,
  configurable: false
});
var desc1 = Object.getOwnPropertyDescriptor(obj, "prop");

try {
  Object.defineProperty(obj, "prop", {
    configurable: true
  });

  throw new Test262Error("Expected TypeError");
} catch (e) {
  assert(e instanceof TypeError);

  assert.sameValue(desc1.configurable, false);
}

verifyProperty(obj, "prop", {
  configurable: false,
});
